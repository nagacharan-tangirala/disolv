use parquet::column::writer::ColumnWriter::FloatColumnWriter;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId};
use disolv_core::bucket::TimeMS;
use disolv_core::hashbrown::HashMap;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};
use disolv_models::device::models::Compose;

use crate::fl::device::DeviceInfo;
use crate::models::ai::data::DataStrategy::Time;
use crate::models::device::message::{
    FlContent, FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit,
};

/// Define a struct that contains details about the data sensors that a device can hold.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct V2XDataSource {
    pub agent_class: AgentClass,
    pub data_size: Bytes,
    pub source_step: TimeMS,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<V2XDataSource>,
    pub size_map: HashMap<FlContent, Bytes>,
}

impl ModelSettings for ComposerSettings {}

#[derive(Debug, Clone)]
pub enum FlComposer {
    Simple(SimpleComposer),
}

impl FlComposer {
    pub fn update_metadata(&mut self, payload: &mut FlPayload, units: &[MessageUnit]) {
        units.iter().for_each(|unit| {
            payload.metadata.total_size += unit.message_size;
            payload.metadata.total_count += 1;
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
            FlComposer::Simple(composer) => &composer.message_to_send,
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
    pub message: Message,
    pub message_type: MessageType,
    pub fl_content: FlContent,
    pub selected_clients: Option<Vec<AgentId>>,
    pub quantity: u64,
}

#[derive(Debug, Clone)]
pub struct SimpleComposer {
    pub data_sources: Vec<V2XDataSource>,
    pub size_map: HashMap<FlContent, Bytes>,
    pub message_to_send: Option<FlMessageDraft>,
    pub step: TimeMS,
}

impl SimpleComposer {
    fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
            size_map: composer_settings.size_map.to_owned(),
            message_to_send: None,
            step: TimeMS::default(),
        }
    }

    fn update_draft(&mut self, message: FlMessageDraft) {
        self.message_to_send = Some(message);
    }

    fn reset_draft(&mut self) {
        self.message_to_send = None;
    }

    fn compose_message_units(&self, target_class: &AgentClass) -> Vec<MessageUnit> {
        let message_units = Vec::new();
        for ds_settings in self.data_sources.iter() {
            if ds_settings.agent_class != *target_class {
                continue;
            }
            if self.step.as_u64() % ds_settings.source_step.as_u64() != TimeMS::default().as_u64() {
                continue;
            }
        }
        message_units
    }

    fn compose_fl_message(
        &self,
        message_draft: FlMessageDraft,
        agent_state: &DeviceInfo,
    ) -> MessageUnit {
        let mut message_size = self
            .size_map
            .get(&message_draft.fl_content)
            .expect("Invalid message")
            .to_owned();

        message_size = Bytes::new(message_size.as_u64() * message_draft.quantity);
        MessageUnit::builder()
            .message_type(message_draft.message_type)
            .message_size(message_size)
            .action(Action::default())
            .fl_content(message_draft.fl_content)
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
            message_units = self.compose_message_units(target_class);
        }

        let mut fl_message = MessageUnit::builder()
            .message_type(MessageType::SensorData)
            .message_size(*self.size_map.get(&FlContent::None).unwrap())
            .action(Action::default())
            .fl_content(FlContent::None)
            .device_info(*agent_state)
            .build();

        if let Some(message) = &self.message_to_send {
            fl_message = self.compose_fl_message(message.to_owned(), agent_state)
        }
        message_units.push(fl_message);

        let payload_info = self.build_metadata(&message_units);
        FlPayload::builder()
            .metadata(payload_info)
            .agent_state(*agent_state)
            .gathered_states(Vec::new())
            .data_units(message_units)
            .query_type(Message::Sensor)
            .build()
    }
}
