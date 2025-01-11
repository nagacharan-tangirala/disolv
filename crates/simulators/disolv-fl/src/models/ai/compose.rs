use hashbrown::HashMap;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};

use crate::fl::device::DeviceInfo;
use crate::models::device::message::{
    FlPayload, FlPayloadInfo, FlTask, Message, MessageType, MessageUnit,
};

/// Define a struct that contains details about the data sensors that a device can hold.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct V2XDataSource {
    pub agent_class: AgentClass,
    pub data_size: Bytes,
    pub sensor_type: SensorType,
    pub source_step: TimeMS,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SensorType {
    Lidar,
    Image,
    Sound,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<V2XDataSource>,
    pub size_map: HashMap<MessageType, Bytes>,
}

impl ModelSettings for ComposerSettings {}

#[derive(Debug, Clone)]
pub enum FlComposer {
    Simple(SimpleComposer),
}

impl FlComposer {
    pub fn encapsulate_units(&mut self, payload: &mut FlPayload, units: &[MessageUnit]) {
        units.iter().for_each(|unit| {
            payload.metadata.total_size += unit.message_size;
            payload.metadata.total_count += 1;
            if unit.fl_task.is_some() {
                payload.query_type = Message::FlMessage;
            }
        });
    }

    pub(crate) fn update_draft(&mut self, message: FlMessageDraft) {
        match self {
            FlComposer::Simple(composer) => composer.update_draft(message),
        }
    }

    pub(crate) fn reset_draft(&mut self) {
        match self {
            FlComposer::Simple(composer) => composer.reset_draft(),
        }
    }

    pub(crate) fn peek_draft(&self) -> &Option<FlMessageDraft> {
        match self {
            FlComposer::Simple(composer) => &composer.message_draft,
        }
    }

    pub(crate) fn compose(
        &self,
        at: TimeMS,
        target_class: &AgentClass,
        agent_state: &DeviceInfo,
    ) -> FlPayload {
        match self {
            FlComposer::Simple(composer) => composer.compose(at, target_class, agent_state),
        }
    }
}

impl Model for FlComposer {
    type Settings = ComposerSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "simple" => FlComposer::Simple(SimpleComposer::new(settings)),
            _ => panic!("Unsupported composer type {}.", settings.name),
        }
    }
}

#[derive(Clone, Default, Debug, TypedBuilder)]
pub struct FlMessageDraft {
    pub message_type: MessageType,
    #[builder(default = None)]
    pub fl_task: Option<FlTask>,
    #[builder(default = None)]
    pub selected_clients: Option<Vec<AgentId>>,
    #[builder(default = 1)]
    pub quantity: u64,
    #[builder(default)]
    pub action_until: TimeMS,
}

#[derive(Debug, Clone)]
pub struct SimpleComposer {
    pub data_sources: Vec<V2XDataSource>,
    pub size_map: HashMap<MessageType, Bytes>,
    pub message_draft: Option<FlMessageDraft>,
    pub step: TimeMS,
}

impl SimpleComposer {
    fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
            size_map: composer_settings.size_map.to_owned(),
            message_draft: None,
            step: TimeMS::default(),
        }
    }

    fn update_draft(&mut self, message: FlMessageDraft) {
        self.message_draft = Some(message);
    }

    fn reset_draft(&mut self) {
        self.message_draft = None;
    }

    fn compose_message_units(
        &self,
        target_class: &AgentClass,
        device_info: DeviceInfo,
    ) -> Vec<MessageUnit> {
        let mut message_units = Vec::new();
        for ds_settings in self.data_sources.iter() {
            if ds_settings.agent_class != *target_class {
                continue;
            }
            if self.step.as_u64() % ds_settings.source_step.as_u64() != TimeMS::default().as_u64() {
                continue;
            }

            let message_unit = MessageUnit::builder()
                .message_type(MessageType::Byte)
                .message(Message::Sensor)
                .message_size(ds_settings.data_size)
                .action(Action::default())
                .device_info(device_info)
                .fl_task(None)
                .build();
            message_units.push(message_unit);
        }
        message_units
    }

    fn compose_fl_message(&self, agent_state: &DeviceInfo) -> MessageUnit {
        let message_draft = self
            .message_draft
            .to_owned()
            .expect("failed to unwrap message draft");
        let mut message_size = self
            .size_map
            .get(&message_draft.message_type)
            .expect("Invalid message")
            .to_owned();

        message_size = Bytes::new(message_size.as_u64() * message_draft.quantity);
        MessageUnit::builder()
            .message(Message::FlMessage)
            .message_type(message_draft.message_type)
            .message_size(message_size)
            .action(Action::default())
            .fl_task(message_draft.fl_task)
            .device_info(*agent_state)
            .build()
    }

    fn build_metadata(&self, data_units: &[MessageUnit]) -> FlPayloadInfo {
        let payload_info = FlPayloadInfo::builder()
            .total_size(data_units.iter().map(|x| x.message_size).sum())
            .total_count(data_units.len() as u32)
            .selected_link(Link::default())
            .build();
        payload_info
    }

    fn compose(
        &self,
        _at: TimeMS,
        target_class: &AgentClass,
        agent_state: &DeviceInfo,
    ) -> FlPayload {
        let mut message_units = Vec::new();
        if !self.data_sources.is_empty() {
            message_units = self.compose_message_units(target_class, *agent_state);
        }

        let mut message = Message::Sensor;

        if let Some(message_draft) = &self.message_draft {
            if message_draft.fl_task.is_some() {
                let fl_message = self.compose_fl_message(agent_state);
                message = Message::FlMessage;
                message_units.push(fl_message);
            }
        }

        let payload_info = self.build_metadata(&message_units);
        FlPayload::builder()
            .metadata(payload_info)
            .agent_state(*agent_state)
            .gathered_states(Vec::new())
            .data_units(message_units)
            .query_type(message)
            .build()
    }
}
