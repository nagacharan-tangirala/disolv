use serde::Deserialize;

use disolv_core::agent::AgentClass;
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};
use disolv_models::device::models::Compose;

use crate::fl::client::AgentInfo;
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
}

impl ModelSettings for ComposerSettings {}

#[derive(Debug, Clone)]
pub enum V2XComposer {
    Sensor(SensorComposer),
}

impl V2XComposer {
    pub fn append_units_to(&mut self, payload: &mut FlPayload, units: &mut Vec<MessageUnit>) {
        units.iter().for_each(|unit| {
            payload.metadata.total_size += unit.message_size;
            payload.metadata.total_count += 1;
        });
        payload.data_units.append(units);
    }
}

impl Model for V2XComposer {
    type Settings = ComposerSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "basic" => V2XComposer::Sensor(SensorComposer::new(settings)),
            _ => panic!("Unsupported composer type {}.", settings.name),
        }
    }
}

impl Compose<MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message> for V2XComposer {
    fn compose(&self, at: TimeMS, target_class: &AgentClass, agent_state: &AgentInfo) -> FlPayload {
        match self {
            V2XComposer::Sensor(composer) => composer.compose(at, target_class, agent_state),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SensorComposer {
    data_sources: Vec<V2XDataSource>,
    step: TimeMS,
}

impl SensorComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
            step: TimeMS::default(),
        }
    }

    fn compose_data_units(&self, target_class: &AgentClass) -> Vec<MessageUnit> {
        let mut message_units = Vec::with_capacity(self.data_sources.len());
        for ds_settings in self.data_sources.iter() {
            if ds_settings.agent_class != *target_class {
                continue;
            }
            if self.step.as_u64() % ds_settings.source_step.as_u64() != TimeMS::default().as_u64() {
                continue;
            }

            let data_blob = MessageUnit::builder()
                .message_type(MessageType::SensorData)
                .message_size(ds_settings.data_size)
                .action(Action::default())
                .fl_content(FlContent::StateInfo)
                .build();
            message_units.push(data_blob);
        }
        message_units
    }

    fn build_metadata(&self, data_units: &Vec<MessageUnit>) -> FlPayloadInfo {
        let payload_info = FlPayloadInfo::builder()
            .total_size(data_units.iter().map(|x| x.message_size).sum())
            .total_count(data_units.len() as u32)
            .selected_link(Link::default())
            .build();
        payload_info
    }
}

impl Compose<MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message> for SensorComposer {
    fn compose(&self, at: TimeMS, target_class: &AgentClass, agent_state: &AgentInfo) -> FlPayload {
        let data_units = self.compose_data_units(target_class);
        let payload_info = self.build_metadata(&data_units);
        FlPayload::builder()
            .metadata(payload_info)
            .agent_state(agent_state.clone())
            .gathered_states(Vec::new())
            .data_units(data_units)
            .query_type(Message::Sensor)
            .build()
    }
}
