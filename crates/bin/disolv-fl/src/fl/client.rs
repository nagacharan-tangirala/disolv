use burn::tensor::backend::AutodiffBackend;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::message::DataUnit;
use disolv_core::radio::ActionType;
use disolv_models::device::actions::am_i_target;
use disolv_models::device::mobility::MapState;
use disolv_models::device::power::PowerState;
use disolv_models::net::radio::CommStats;

use crate::fl::bucket::FlBucket;
use crate::fl::device::DeviceInfo;
use crate::models::ai::compose::FlMessageDraft;
use crate::models::ai::data::DataHolder;
use crate::models::ai::models::{
    ClientState, ModelDirection, ModelLevel, ModelType, TrainingStatus,
};
use crate::models::ai::times::ClientTimes;
use crate::models::ai::trainer::Trainer;
use crate::models::device::message::{FlContent, FlPayload, Message, MessageType};
use crate::models::device::output::ModelUpdate;

#[derive(Clone, TypedBuilder)]
pub(crate) struct ClientModels<B: AutodiffBackend> {
    pub(crate) trainer: Trainer<B>,
    pub(crate) local_model: ModelType<B>,
    pub(crate) times: ClientTimes,
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
}

impl<B: AutodiffBackend> Client<B> {
    pub(crate) fn init(&mut self, bucket: &mut FlBucket<B>) {
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
    }

    pub(crate) fn handle_incoming(&mut self, bucket: &mut FlBucket<B>, payloads: &[FlPayload]) {
        for payload in payloads.iter() {
            for message_unit in payload.data_units.iter() {
                if message_unit.fl_content == FlContent::None {
                    continue;
                }

                if !am_i_target(message_unit.action(), &self.client_info) {
                    debug!("{:?} is not for me", message_unit);
                    continue;
                }
                debug!("Got an FL Message for agent {}", self.client_info.id);
                match message_unit.fl_content {
                    FlContent::StateInfo => self.prepare_state_update(bucket),
                    FlContent::CompleteTraining => self.complete_training(bucket),
                    FlContent::GlobalModel => self.collect_global_model(bucket),
                    FlContent::ClientSelected => self.initiate_training(bucket),
                    _ => panic!("Client should not receive this message"),
                };
                // This is to ensure that client only listens to the first FL related instruction.
                return;
            }
        }
    }

    pub(crate) fn draft_fl_message(&self, _bucket: &mut FlBucket<B>) -> FlMessageDraft {}

    fn prepare_state_update(&mut self, bucket: &mut FlBucket<B>) {
        self.client_state = ClientState::Preparing;
        self.message_draft = FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .selected_clients(None)
            .build();
        self.write_state_update(bucket);
    }

    fn collect_global_model(&mut self, bucket: &mut FlBucket<B>) {
        self.fl_models.local_model = match bucket.models.model_lake.global_model.to_owned() {
            Some(val) => val,
            None => panic!("Global model not present"),
        };
        self.message_draft = FlMessageDraft::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .selected_clients(None)
            .build();

        self.write_model_update(
            bucket,
            AgentId::from(1000000),
            self.client_info.id,
            ModelLevel::Global,
            ModelDirection::Received,
            TrainingStatus::NA,
            -1.0,
        );
    }

    fn initiate_training(&mut self, bucket: &mut FlBucket<B>) {
        self.client_state = ClientState::Training;
        self.fl_models
            .times
            .update_time(self.step, self.client_state);

        self.fl_models.trainer.train_data = self.fl_models.holder.allotted_train_data();
        self.fl_models.trainer.test_data = self.fl_models.holder.allotted_test_data();

        self.fl_models.trainer.train(&bucket.models.device);
        self.fl_models.trainer.save_model_to_file(self.step);
        self.fl_models.local_model = self.fl_models.trainer.model.to_owned();
        self.write_state_update(bucket);
    }

    fn complete_training(&mut self, bucket: &mut FlBucket<B>) {
        self.client_state = ClientState::Sensing;
        debug!("Completed training in {}", self.client_info.id);

        if self.is_training_complete() {
            let accuracy = self.fl_models.trainer.test_model(&bucket.models.device);
            self.message_draft.message = Message::FlMessage;
            self.message_draft.fl_content = FlContent::LocalModel;
            self.message_draft.message_type = MessageType::F64Weights;
            self.message_draft.quantity = self.fl_models.trainer.no_of_weights;
            bucket
                .models
                .model_lake
                .add_local_model(self.client_info.id, self.fl_models.trainer.model.clone());

            self.write_model_update(
                bucket,
                self.client_info.id,
                AgentId::from(1000000),
                ModelLevel::Local,
                ModelDirection::Sent,
                TrainingStatus::Success,
                accuracy,
            );
        } else {
            self.message_draft.message = Message::FlMessage;
            self.message_draft.fl_content = FlContent::TrainingFailed;
            self.message_draft.message_type = MessageType::KiloByte;
            self.message_draft.quantity = 1;

            self.write_model_update(
                bucket,
                self.client_info.id,
                AgentId::from(1000000),
                ModelLevel::Local,
                ModelDirection::NA,
                TrainingStatus::Failure,
                -1.0,
            );
        }
        self.write_state_update(bucket);
    }

    fn is_training_complete(&self) -> bool {
        // todo: add dynamic training times considering hardware usage etc.
        true
    }

    fn write_state_update(&self, bucket: &mut FlBucket<B>) {
        if let Some(writer) = &mut bucket.models.output.fl_state_writer {
            writer.add_data(
                self.step,
                self.client_info.id,
                self.client_state.to_string(),
            );
        }
    }

    fn write_model_update(
        &self,
        bucket: &mut FlBucket<B>,
        source_id: AgentId,
        target_id: AgentId,
        level: ModelLevel,
        direction: ModelDirection,
        status: TrainingStatus,
        accuracy: f32,
    ) {
        if let Some(writer) = &mut bucket.models.output.fl_model_writer {
            let model_update = ModelUpdate::builder()
                .time_step(self.step)
                .agent_id(source_id)
                .target_id(target_id)
                .agent_state(self.client_state.to_string())
                .model(level.to_string())
                .direction(direction.to_string())
                .status(status.to_string())
                .accuracy(accuracy)
                .build();

            writer.add_data(model_update);
        }
    }
}
