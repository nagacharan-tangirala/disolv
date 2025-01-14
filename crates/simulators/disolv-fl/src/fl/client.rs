use burn::tensor::backend::AutodiffBackend;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::message::DataUnit;
use disolv_models::device::actions::am_i_target;
use disolv_output::tables::model::ModelUpdate;
use disolv_output::tables::train::FlTrainingData;

use crate::fl::bucket::FlBucket;
use crate::fl::device::DeviceInfo;
use crate::models::ai::compose::FlMessageDraft;
use crate::models::ai::data::DataHolder;
use crate::models::ai::models::{ClientState, ModelDirection, ModelLevel, TrainingStatus};
use crate::models::ai::trainer::Trainer;
use crate::models::device::message::{FlPayload, FlTask, Message, MessageType};

#[derive(Clone, TypedBuilder)]
pub(crate) struct ClientModels<B: AutodiffBackend> {
    pub(crate) trainer: Trainer<B>,
    pub(crate) holder: DataHolder,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Client<B: AutodiffBackend> {
    pub(crate) client_info: DeviceInfo,
    pub(crate) fl_models: ClientModels<B>,
    #[builder(default)]
    pub(crate) message_draft: FlMessageDraft,
    #[builder(default)]
    pub(crate) client_state: ClientState,
    #[builder(default)]
    pub(crate) step: TimeMS,
    #[builder(default)]
    pub(crate) server_id: AgentId,
    #[builder(default)]
    pub(crate) draft_change_at: TimeMS,
}

impl<B: AutodiffBackend> Client<B> {
    pub(crate) fn init(&mut self, bucket: &mut FlBucket<B>) {
        self.step = bucket.step;
        self.fl_models.trainer.train_data = bucket
            .training_data_for(self.client_info.id)
            .expect("no training data set for this agent");
        self.fl_models.trainer.test_data = bucket
            .testing_data_for(self.client_info.id)
            .expect("no test data set for this agent");

        self.fl_models
            .holder
            .set_train_data(self.fl_models.trainer.train_data.to_owned());
        self.fl_models
            .holder
            .set_test_data(self.fl_models.trainer.test_data.to_owned());
        self.client_state = ClientState::Sensing;
        self.write_state_update(bucket);
    }

    pub(crate) fn update_step(&mut self, new_step: TimeMS) {
        self.step = new_step;
        self.fl_models.holder.allot_data(self.step);
    }

    pub(crate) fn handle_incoming(&mut self, bucket: &mut FlBucket<B>, payloads: &[FlPayload]) {
        let mut got_fl_message = false;
        for payload in payloads.iter() {
            if payload.query_type != Message::FlMessage {
                continue;
            }

            got_fl_message = true;
            payload.data_units.iter().for_each(|message_unit| {
                if am_i_target(message_unit.action(), &self.client_info) {
                    if let Some(ref fl_task) = message_unit.fl_task {
                        self.do_fl_task(fl_task, bucket);
                    }
                }
            });
        }
        if !got_fl_message {
            self.check_draft_validity(bucket);
        }
    }

    pub(crate) fn do_fl_task(&mut self, fl_task: &FlTask, bucket: &mut FlBucket<B>) {
        match (fl_task, self.client_state) {
            (FlTask::StateRequest(time_limit), ClientState::Sensing) => {
                self.draft_change_at = *time_limit;
                self.prepare_state_update(bucket);
            }
            (FlTask::GlobalModel(server_id), ClientState::Informing) => {
                self.server_id = *server_id;
                self.collect_global_model(bucket);
            }
            (FlTask::RoundBegin, ClientState::ReadyToTrain) => self.initiate_training(bucket),
            (FlTask::RoundComplete(time_limit), ClientState::Training) => {
                self.draft_change_at = *time_limit;
                self.complete_training(bucket);
            }
            _ => {}
        };
    }

    pub(crate) fn draft_fl_message(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        self.message_draft.clone()
    }

    fn check_draft_validity(&mut self, bucket: &mut FlBucket<B>) {
        if self.step > self.draft_change_at {
            self.message_draft = FlMessageDraft::builder()
                .message_type(MessageType::KiloByte)
                .build();
            if self.client_state == ClientState::Informing {
                self.client_state = ClientState::Sensing;
                self.write_state_update(bucket);
            }
        }
    }

    fn prepare_state_update(&mut self, bucket: &mut FlBucket<B>) {
        self.message_draft = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .fl_task(Some(FlTask::StateInfo))
            .build();

        self.client_state = ClientState::Informing;
        self.write_state_update(bucket);
    }

    fn collect_global_model(&mut self, bucket: &mut FlBucket<B>) {
        self.fl_models.trainer.model = match bucket.models.model_lake.global_model.to_owned() {
            Some(val) => val,
            None => panic!("Global model not present"),
        };
        self.client_state = ClientState::ReadyToTrain;
        self.write_state_update(bucket);

        self.message_draft = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .build();

        if let Some(writer) = &mut bucket.models.results.model {
            let model_update = ModelUpdate::builder()
                .time_step(self.step.as_u64())
                .agent_id(self.server_id.as_u64())
                .target_id(self.client_info.id.as_u64())
                .agent_state(self.client_state.to_string())
                .direction(ModelDirection::Received.to_string())
                .build();
            writer.add_data(model_update);
        }
    }

    fn initiate_training(&mut self, bucket: &mut FlBucket<B>) {
        self.client_state = ClientState::Training;
        self.fl_models.trainer.train_data = self.fl_models.holder.allotted_train_data();
        self.fl_models.trainer.test_data = self.fl_models.holder.allotted_test_data();

        let train_data = FlTrainingData::builder()
            .agent_id(self.client_info.id.as_u64())
            .train_len(self.fl_models.trainer.train_data.data_length() as u32)
            .test_len(self.fl_models.trainer.test_data.data_length() as u32)
            .build();

        if let Some(writer) = &mut bucket.models.results.train {
            writer.add_data(self.step, train_data);
        }

        self.fl_models.trainer.train(&bucket.models.device);
        self.fl_models.trainer.save_model_to_file(self.step);

        self.message_draft = FlMessageDraft::builder()
            .message_type(MessageType::KiloByte)
            .build();
        self.write_state_update(bucket);
    }

    fn complete_training(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Completed training in {}", self.client_info.id);

        // Initiate variables in case of failure.
        self.message_draft.message_type = MessageType::KiloByte;
        self.message_draft.quantity = 1;

        let model_level = ModelLevel::Local;
        let mut direction = ModelDirection::NA;
        let mut status = TrainingStatus::Failure;
        let mut accuracy = -1.0;

        if self.is_training_complete() {
            accuracy = self.fl_models.trainer.test_model(&bucket.models.device);
            self.upload_local_model(bucket);
            direction = ModelDirection::Sent;
            status = TrainingStatus::Success;
        }

        if let Some(writer) = &mut bucket.models.results.model {
            let model_update = ModelUpdate::builder()
                .time_step(self.step.as_u64())
                .agent_id(self.client_info.id.as_u64())
                .target_id(self.server_id.as_u64())
                .agent_state(self.client_state.to_string())
                .model(model_level.to_string())
                .direction(direction.to_string())
                .status(status.to_string())
                .accuracy(accuracy)
                .build();

            writer.add_data(model_update);
        }

        self.client_state = ClientState::Sensing;
        self.write_state_update(bucket);
    }

    fn upload_local_model(&mut self, bucket: &mut FlBucket<B>) {
        self.message_draft = FlMessageDraft::builder()
            .fl_task(Some(FlTask::LocalModel))
            .message_type(MessageType::F64Weight)
            .quantity(self.fl_models.trainer.no_of_weights)
            .build();

        bucket
            .models
            .model_lake
            .add_local_model(self.client_info.id, self.fl_models.trainer.model.clone());
    }

    fn is_training_complete(&self) -> bool {
        // todo: add dynamic training times considering hardware usage etc.
        true
    }

    fn write_state_update(&self, bucket: &mut FlBucket<B>) {
        if let Some(writer) = &mut bucket.models.results.state {
            writer.add_data(
                self.step,
                self.client_info.id,
                self.client_state.to_string(),
            );
        }
    }
}
