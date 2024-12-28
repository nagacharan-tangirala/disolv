use log::{debug, error};
use serde::Deserialize;

use disolv_core::agent::AgentClass;
use disolv_core::bucket::TimeMS;
use disolv_core::message::Payload;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};
use disolv_models::device::models::Compose;

use crate::models::message::{DataBlob, DataType, MessageType, PayloadInfo, V2XPayload};
use crate::v2x::device::DeviceInfo;

/// Define a struct that contains details about the data sensors that a device can hold.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub agent_class: AgentClass,
    pub data_size: Bytes,
    pub source_step: TimeMS,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<DataSource>,
}

impl ModelSettings for ComposerSettings {}

#[derive(Clone, Debug)]
pub enum Composer {
    Basic(BasicComposer),
}

impl Model for Composer {
    type Settings = ComposerSettings;

    fn with_settings(settings: &ComposerSettings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "basic" => Composer::Basic(BasicComposer::new(settings)),
            _ => {
                error!("Only Basic composer is supported.");
                panic!("Unsupported composer type {}.", settings.name);
            }
        }
    }
}

impl Composer {
    pub fn append_blobs_to(&mut self, payload: &mut V2XPayload, units: &mut Vec<DataBlob>) {
        units.iter().for_each(|unit| {
            payload.metadata.total_size += unit.data_size;
            payload.metadata.total_count += 1;
        });
        payload.data_units.append(units);
    }
}

impl Compose<DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType> for Composer {
    fn compose(
        &self,
        at: TimeMS,
        target_class: &AgentClass,
        agent_state: &DeviceInfo,
    ) -> V2XPayload {
        match self {
            Composer::Basic(composer) => composer.compose(at, target_class, agent_state),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BasicComposer {
    pub data_sources: Vec<DataSource>,
}

impl BasicComposer {
    fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
        }
    }

    fn compose_data_units(&self, step: TimeMS, target_class: &AgentClass) -> Vec<DataBlob> {
        let mut data_blobs = Vec::with_capacity(self.data_sources.len());
        for ds_settings in self.data_sources.iter() {
            if ds_settings.agent_class != *target_class {
                continue;
            }
            if step.as_u64() % ds_settings.source_step.as_u64() != TimeMS::default().as_u64() {
                continue;
            }

            let data_blob = DataBlob::builder()
                .data_type(ds_settings.data_type)
                .data_size(ds_settings.data_size)
                .action(Action::default())
                .build();
            data_blobs.push(data_blob);
        }
        data_blobs
    }

    fn build_metadata(&self, data_units: &Vec<DataBlob>) -> PayloadInfo {
        let payload_info = PayloadInfo::builder()
            .id(uuid::Uuid::new_v4())
            .total_size(data_units.iter().map(|x| x.data_size).sum())
            .total_count(data_units.len() as u32)
            .selected_link(Link::default())
            .build();
        payload_info
    }
}

impl Compose<DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType> for BasicComposer {
    fn compose(
        &self,
        at: TimeMS,
        target_class: &AgentClass,
        agent_state: &DeviceInfo,
    ) -> V2XPayload {
        let data_units = self.compose_data_units(at, target_class);
        let payload_info = self.build_metadata(&data_units);
        V2XPayload::builder()
            .metadata(payload_info)
            .agent_state(*agent_state)
            .gathered_states(Vec::new())
            .data_units(data_units)
            .query_type(MessageType::Sensor)
            .build()
    }
}

// #[derive(Clone, Debug)]
// pub struct CachedComposer {
//     pub data_sources: Vec<DataSource>,
//     pub data_cache: VecDeque<DataBlob>,
//     pub last_contacted: TimeMS,
// }
//
// impl CachedComposer {
//     pub fn new(composer_settings: &ComposerSettings) -> Self {
//         Self {
//             data_sources: composer_settings.source_settings.to_owned(),
//             data_cache: VecDeque::new(),
//             last_contacted: TimeMS::default(),
//         }
//     }
//
//     fn compose_payload(&self, target_class: &AgentClass, content: DeviceContent) -> DPayload {
//         let payload_info = self.compose_metadata(target_class);
//         DPayload::builder()
//             .metadata(payload_info)
//             .agent_state(content)
//             .gathered_states(Some(Vec::new()))
//             .build()
//     }
// }
