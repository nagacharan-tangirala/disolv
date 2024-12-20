use std::fmt::{Display, Formatter};

use burn::tensor::backend::AutodiffBackend;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId};
use disolv_core::bucket::TimeMS;
use disolv_core::radio::ActionType;

use crate::fl::bucket::FlBucket;
use crate::fl::device::DeviceInfo;
use crate::models::ai::aggregate::Aggregator;
use crate::models::ai::compose::FlMessageDraft;
use crate::models::ai::data::DataHolder;
use crate::models::ai::models::{ModelDirection, ModelLevel, TrainingStatus};
use crate::models::ai::select::ClientSelector;
use crate::models::ai::times::ServerTimes;
use crate::models::ai::trainer::Trainer;
use crate::models::device::message::{FlContent, FlPayload, Message, MessageType};
use crate::models::device::output::ModelUpdate;

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

        self.fl_models.trainer.test_data = bucket.models.data_distributor.server_test_data();

        self.fl_models
            .holder
            .set_test_data(self.fl_models.trainer.test_data.to_owned());

        self.fl_models.holder.allot_data();
        bucket.models.model_lake.global_model = Some(self.fl_models.trainer.model.clone());
    }

    pub(crate) fn draft_fl_message(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        let mut message_draft = FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::None)
            .selected_clients(None)
            .build();

        if self.fl_models.times.is_time_to_change(self.step) {
            message_draft = match self.server_state {
                ServerState::Idle => self.handle_initiation(bucket),
                ServerState::ClientAnalysis => self.handle_analysis(bucket),
                ServerState::ClientSelection => self.handle_selection(bucket),
                ServerState::TrainingRound => self.handle_training(bucket),
                ServerState::Aggregation => self.handle_aggregation(bucket),
            };
            self.fl_models
                .times
                .update_time(self.step, self.server_state);
            self.write_state_update(bucket);
        } else {
            match self.server_state {
                ServerState::Idle => {}
                ServerState::ClientAnalysis => message_draft.fl_content = FlContent::StateInfo,
                ServerState::ClientSelection => {
                    message_draft.message_type = MessageType::KiloByte;
                    message_draft.fl_content = FlContent::ClientSelected;
                    message_draft.selected_clients =
                        Some(self.fl_models.client_selector.selected_clients().clone());
                }
                ServerState::TrainingRound => {}
                ServerState::Aggregation => {}
            };
        }
        message_draft
    }

    pub(crate) fn handle_incoming(&mut self, bucket: &mut FlBucket<B>, payloads: &[FlPayload]) {
        for payload in payloads.iter() {
            payload
                .data_units
                .iter()
                .for_each(|data_unit| match data_unit.fl_content {
                    FlContent::None => {}
                    FlContent::StateInfo => self.register_client(&data_unit.device_info),
                    FlContent::LocalModel => {
                        self.collect_local_model(bucket, &data_unit.device_info)
                    }
                    FlContent::TrainingFailed => debug!("a client failed to train"),
                    _ => panic!("Server should not receive this message"),
                });
            // payload
            //     .data_units
            //     .iter_mut()
            //     .for_each(|message| message.action.action_type = ActionType::Consume);
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

        debug!("Collecting local model of {}", client_info.id);
        let local_model = bucket.models.model_lake.local_model_of(client_info.id);
        self.fl_models
            .aggregator
            .add_local_model(local_model, client_info.id);
        self.write_model_update(
            bucket,
            client_info.id,
            self.server_info.id,
            ModelLevel::Local,
            ModelDirection::Received,
            TrainingStatus::NA,
            -1.0,
        );
    }

    fn handle_initiation(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from idle to analysis at {}", self.step);
        self.server_state = ServerState::ClientAnalysis;

        FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .selected_clients(None)
            .build()
    }

    fn handle_analysis(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        let mut message_to_build = FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .selected_clients(None)
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

                message_to_build.message_type = MessageType::KiloByte;
                message_to_build.fl_content = FlContent::ClientSelected;
                message_to_build.selected_clients = Some(selected_clients);
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

        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        debug!("{} is the selected client count", selected_clients.len());

        FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::GlobalModel)
            .selected_clients(Some(selected_clients))
            .build()
    }

    fn handle_training(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from training to aggregation at {}", self.step);
        self.server_state = ServerState::Aggregation;

        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();

        FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::CompleteTraining)
            .selected_clients(Some(selected_clients))
            .build()
    }

    fn handle_aggregation(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        debug!("Changing from aggregation to idle at {}", self.step);
        self.server_state = ServerState::Idle;

        let current_global_model = self.fl_models.trainer.model.clone();
        self.fl_models.trainer.model = self
            .fl_models
            .aggregator
            .aggregate(current_global_model, &bucket.models.device);
        self.fl_models.aggregator.clear_local_models();

        self.fl_models.trainer.save_model_to_file(self.step);
        bucket
            .models
            .model_lake
            .update_global_model(self.fl_models.trainer.model.clone(), self.step);

        let model_accuracy = self.fl_models.trainer.test_model(&bucket.models.device);
        self.write_model_update(
            bucket,
            AgentId::from(100000),
            AgentId::from(100000),
            ModelLevel::Global,
            ModelDirection::Received,
            TrainingStatus::NA,
            model_accuracy,
        );

        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::None)
            .selected_clients(Some(selected_clients))
            .build()
    }

    fn write_state_update(&self, bucket: &mut FlBucket<B>) {
        if let Some(writer) = &mut bucket.models.output.fl_state_writer {
            writer.add_data(
                self.step,
                self.server_info.id,
                self.server_state.to_string(),
            );
        }
    }

    fn write_model_update(
        &self,
        bucket: &mut FlBucket<B>,
        source: AgentId,
        target: AgentId,
        level: ModelLevel,
        direction: ModelDirection,
        status: TrainingStatus,
        accuracy: f32,
    ) {
        if let Some(writer) = &mut bucket.models.output.fl_model_writer {
            let model_update = ModelUpdate::builder()
                .time_step(self.step)
                .agent_id(source)
                .target_id(target)
                .agent_state(self.server_state.to_string())
                .model(level.to_string())
                .direction(direction.to_string())
                .status(status.to_string())
                .accuracy(accuracy)
                .build();

            writer.add_data(model_update);
        }
    }
}
