use std::arch::aarch64::float32x2_t;

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

use crate::fl::client::AgentInfo;
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
    pub fn append_units_to(&mut self, payload: &mut FlPayload, units: &mut Vec<MessageUnit>) {
        units.iter().for_each(|unit| {
            payload.metadata.total_size += unit.message_size;
            payload.metadata.total_count += 1;
        });
        payload.data_units.append(units);
    }

    pub(crate) fn set_message_to_build(&mut self, message: FlMessageToBuild) {
        match self {
            FlComposer::Simple(composer) => composer.set_message_to_build(message),
        }
    }

    pub(crate) fn compose(
        &mut self,
        at: TimeMS,
        target_class: &AgentClass,
        agent_state: &AgentInfo,
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

#[derive(Clone, Default, Copy, Debug, TypedBuilder)]
pub struct FlMessageToBuild {
    pub message: Message,
    pub message_type: MessageType,
    pub fl_content: FlContent,
    pub quantity: u64,
}

#[derive(Debug, Clone)]
pub struct SimpleComposer {
    pub data_sources: Vec<V2XDataSource>,
    pub size_map: HashMap<FlContent, Bytes>,
    pub message_to_send: Option<FlMessageToBuild>,
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

    fn set_message_to_build(&mut self, message: FlMessageToBuild) {
        self.message_to_send = Some(message);
    }

    fn compose_message_units(&self, target_class: &AgentClass) -> Vec<MessageUnit> {
        let mut message_units = Vec::new();
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

    fn compose_fl_message(&self, message_to_build: FlMessageToBuild) -> MessageUnit {
        let mut message_size = self
            .size_map
            .get(&message_to_build.fl_content)
            .expect("Invalid message")
            .to_owned();

        message_size = Bytes::new(message_size.as_u64() * message_to_build.quantity);
        MessageUnit::builder()
            .message_type(message_to_build.message_type.clone())
            .message_size(message_size)
            .action(Action::default())
            .fl_content(message_to_build.fl_content)
            .build()
    }

    fn build_metadata(&self, data_units: &Vec<MessageUnit>) -> FlPayloadInfo {
        let payload_info = FlPayloadInfo::builder()
            .total_size(data_units.iter().map(|x| x.message_size).sum())
            .total_count(data_units.len() as u32)
            .selected_link(Link::default())
            .build();
        payload_info
    }

    fn compose(
        &mut self,
        _at: TimeMS,
        target_class: &AgentClass,
        agent_state: &AgentInfo,
    ) -> FlPayload {
        let mut message_units = Vec::new();
        if self.data_sources.len() > 0 {
            message_units = self.compose_message_units(target_class);
        }

        let mut fl_message = MessageUnit::builder()
            .message_type(MessageType::SensorData)
            .message_size(*self.size_map.get(&FlContent::None).unwrap())
            .action(Action::default())
            .fl_content(FlContent::None)
            .build();

        if let Some(message) = self.message_to_send {
            fl_message = self.compose_fl_message(message);
            self.message_to_send = None;
        }
        message_units.push(fl_message);

        let payload_info = self.build_metadata(&message_units);
        FlPayload::builder()
            .metadata(payload_info)
            .agent_state(agent_state.clone())
            .gathered_states(Vec::new())
            .data_units(message_units)
            .query_type(Message::Sensor)
            .build()
    }
}
