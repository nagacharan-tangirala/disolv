use std::fmt::{Display, Formatter};

use burn::tensor::backend::AutodiffBackend;
use log::{debug, info};
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentClass;
use disolv_core::bucket::TimeMS;
use disolv_output::tables::model::ModelUpdate;
use disolv_output::tables::select::ClientSelectData;

use crate::fl::bucket::FlBucket;
use crate::fl::device::DeviceInfo;
use crate::models::ai::aggregate::Aggregator;
use crate::models::ai::common::{ModelDirection, ModelLevel};
use crate::models::ai::compose::FlMessageDraft;
use crate::models::ai::model::ModelType;
use crate::models::ai::select::ClientSelector;
use crate::models::ai::times::ServerTimes;
use crate::models::ai::trainer::Trainer;
use crate::models::data::allot::DataHolder;
use crate::models::device::message::{FlPayload, FlTask, MessageType};

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerState {
    #[default]
    Idle,
    ClientAnalysis,
    ClientSelection,
    TrainingRound,
    Aggregation,
}

impl Display for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerState::Idle => write!(f, "Idle"),
            ServerState::ClientAnalysis => write!(f, "ClientAnalysis"),
            ServerState::ClientSelection => write!(f, "ClientSelection"),
            ServerState::TrainingRound => write!(f, "TrainingRound"),
            ServerState::Aggregation => write!(f, "Aggregation"),
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct FlServerModels<B: AutodiffBackend> {
    pub(crate) client_classes: Vec<AgentClass>,
    pub(crate) trainer: Trainer<B>,
    pub(crate) client_selector: ClientSelector,
    pub(crate) times: ServerTimes,
    pub(crate) aggregator: Aggregator<B>,
    pub(crate) holder: DataHolder,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Server<B: AutodiffBackend> {
    pub(crate) server_info: DeviceInfo,
    pub(crate) fl_models: FlServerModels<B>,
    #[builder(default)]
    pub(crate) server_state: ServerState,
    #[builder(default)]
    pub(crate) step: TimeMS,
}

impl<B: AutodiffBackend> Server<B> {
    pub(crate) fn init(&mut self, bucket: &mut FlBucket<B>) {
        self.fl_models
            .times
            .update_time(self.step, self.server_state);

        let test_data = match &self.fl_models.trainer.model {
            ModelType::Mnist(_) => bucket.models.data_distributor.server_test_data(),
            ModelType::Cifar(_) => bucket.models.data_distributor.server_test_data(),
        };

        self.fl_models.holder.set_test_data(test_data);

        self.fl_models.holder.allot_data(self.step);
        bucket.models.model_lake.global_model = Some(self.fl_models.trainer.model.clone());
    }

    pub(crate) fn update_step(&mut self, new_step: TimeMS) {
        self.step = new_step;
    }

    pub(crate) fn draft_fl_message(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        let mut message_draft = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .build();

        if self.fl_models.times.is_time_to_change(self.step) {
            message_draft = match self.server_state {
                ServerState::Idle => self.handle_initiation(bucket),
                ServerState::ClientAnalysis => self.handle_analysis(bucket),
                ServerState::ClientSelection => self.handle_selection(bucket),
                ServerState::TrainingRound => self.handle_training(bucket),
                ServerState::Aggregation => self.handle_aggregation(bucket),
            };
            self.write_state_update(bucket);
        } else {
            match self.server_state {
                ServerState::Idle => {}
                ServerState::ClientAnalysis => {
                    message_draft.fl_task = Some(FlTask::StateRequest(
                        self.step + self.fl_models.times.durations.analysis,
                    ))
                }
                ServerState::ClientSelection => {
                    message_draft.message_type = MessageType::F64Weight;
                    message_draft.fl_task = Some(FlTask::GlobalModel(self.server_info.id));
                    message_draft.selected_clients =
                        Some(self.fl_models.client_selector.selected_clients().clone());
                    message_draft.quantity = self.fl_models.trainer.no_of_weights;
                }
                ServerState::TrainingRound => {}
                ServerState::Aggregation => {}
            };
        }
        message_draft
    }

    pub(crate) fn handle_incoming(&mut self, bucket: &mut FlBucket<B>, payloads: &[FlPayload]) {
        for payload in payloads.iter() {
            payload.data_units.iter().for_each(|message_unit| {
                if let Some(ref fl_task) = message_unit.fl_task {
                    match fl_task {
                        FlTask::StateInfo => self.register_client(&message_unit.device_info),
                        FlTask::LocalModel => {
                            self.collect_local_model(bucket, &message_unit.device_info)
                        }
                        _ => panic!("Server should not receive this message"),
                    }
                }
            });
        }
    }

    fn register_client(&mut self, agent_info: &DeviceInfo) {
        if self
            .fl_models
            .client_classes
            .contains(&agent_info.agent_class)
        {
            self.fl_models.client_selector.register_client(agent_info);
        }
    }

    fn collect_local_model(&mut self, bucket: &mut FlBucket<B>, client_info: &DeviceInfo) {
        if self.fl_models.aggregator.is_model_collected(client_info.id) {
            return;
        }
        if self.server_state != ServerState::Aggregation {
            return;
        }

        info!("Collecting local model of {}", client_info.id);
        let local_model = bucket.models.model_lake.local_model_of(client_info.id);
        self.fl_models
            .aggregator
            .add_local_model(local_model, client_info.id);

        if let Some(writer) = &mut bucket.models.results.model {
            let model_update = ModelUpdate::builder()
                .time_step(self.step.as_u64())
                .agent_id(client_info.id.as_u64())
                .target_id(self.server_info.id.as_u64())
                .agent_state(self.server_state.to_string())
                .model(ModelLevel::Local.to_string())
                .direction(ModelDirection::Received.to_string())
                .build();
            writer.add_data(model_update);
        }
    }

    fn handle_initiation(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from idle to analysis at {}", self.step);
        self.server_state = ServerState::ClientAnalysis;
        self.fl_models.client_selector.clear_states();

        self.fl_models
            .times
            .update_time(self.step, self.server_state);

        FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .fl_task(Some(FlTask::StateRequest(
                self.step + self.fl_models.times.durations.analysis,
            )))
            .build()
    }

    fn handle_analysis(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        let mut message_to_build = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .build();

        if self.fl_models.client_selector.has_clients() {
            self.fl_models.client_selector.do_selection();

            if !self.fl_models.client_selector.selected_clients().is_empty() {
                debug!("Changing from analysis to selection at {}", self.step);
                self.fl_models
                    .times
                    .update_time(self.step, self.server_state);

                self.server_state = ServerState::ClientSelection;
                let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();

                if let Some(writer) = &mut bucket.models.results.select {
                    let select_data = ClientSelectData::builder()
                        .server_id(self.server_info.id.as_u64())
                        .available(self.fl_models.client_selector.registered_count() as u32)
                        .selected(self.fl_models.client_selector.selected_as_string())
                        .build();
                    writer.add_data(self.step, select_data);
                }

                self.fl_models
                    .times
                    .update_time(self.step, self.server_state);

                message_to_build.message_type = MessageType::F64Weight;
                message_to_build.fl_task = Some(FlTask::GlobalModel(self.server_info.id));
                message_to_build.selected_clients = Some(selected_clients);
                message_to_build.quantity = self.fl_models.trainer.no_of_weights;
            } else {
                debug!("No clients selected. Continuing analysis at {}", self.step);
            }
        } else {
            debug!(
                "No clients registered. Continuing analysis at {}",
                self.step
            );
        }
        message_to_build
    }

    fn handle_selection(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from selection to training at {}", self.step);
        self.server_state = ServerState::TrainingRound;

        self.fl_models
            .times
            .update_time(self.step, self.server_state);
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        debug!("{} is the selected client count", selected_clients.len());

        FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .fl_task(Some(FlTask::RoundBegin))
            .selected_clients(Some(selected_clients))
            .build()
    }

    fn handle_training(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from training to aggregation at {}", self.step);
        self.server_state = ServerState::Aggregation;

        self.fl_models
            .times
            .update_time(self.step, self.server_state);
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        debug!("Selected clients are {:?}", selected_clients);

        FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .fl_task(Some(FlTask::RoundComplete(
                self.step + self.fl_models.times.durations.aggregation,
            )))
            .selected_clients(Some(selected_clients))
            .build()
    }

    fn handle_aggregation(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from aggregation to idle at {}", self.step);
        self.server_state = ServerState::Idle;

        self.fl_models
            .times
            .update_time(self.step, self.server_state);

        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        let message_draft = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .selected_clients(Some(selected_clients))
            .build();

        // If there are no local models, return the draft without updating global model.
        if !self.fl_models.aggregator.has_local_models() {
            return message_draft;
        }

        self.fl_models
            .aggregator
            .aggregate(&mut self.fl_models.trainer.model, &bucket.models.device);

        self.fl_models.holder.allot_data(self.step);
        let test_data = self.fl_models.holder.allotted_test_data();

        let model_accuracy = self
            .fl_models
            .trainer
            .test_model(&bucket.models.device, test_data);

        bucket
            .models
            .model_lake
            .update_global_model(self.fl_models.trainer.model.clone(), self.step);
        self.fl_models.trainer.save_model_to_file(self.step);

        if let Some(writer) = &mut bucket.models.results.model {
            let model_update = ModelUpdate::builder()
                .time_step(self.step.as_u64())
                .agent_id(self.server_info.id.as_u64())
                .target_id(self.server_info.id.as_u64())
                .agent_state(self.server_state.to_string())
                .accuracy(model_accuracy)
                .build();
            writer.add_data(model_update);
        }
        message_draft
    }

    fn write_state_update(&self, bucket: &mut FlBucket<B>) {
        if let Some(writer) = &mut bucket.models.results.state {
            writer.add_data(
                self.step,
                self.server_info.id,
                self.server_state.to_string(),
            );
        }
    }
}
