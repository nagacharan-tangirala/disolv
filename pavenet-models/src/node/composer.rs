use crate::model::{NodeModel, TomlReadable};
use crate::node::payload::{Payload, SensorData};
use pavenet_core::enums::{DataType, NodeType};
use serde::Deserialize;

pub const SOURCE_SIZE: usize = 10;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub target_type: NodeType,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<DataSource>,
}

impl TomlReadable for ComposerSettings {}

impl ComposerSettings {
    pub fn ds_array(&self) -> [Option<DataSource>; SOURCE_SIZE] {
        let mut data_sources = [None; SOURCE_SIZE];
        for idx in 0..self.source_settings.len() {
            data_sources[idx] = Some(self.source_settings[idx]);
        }
        return data_sources;
    }

    pub fn ds_count(&self) -> usize {
        self.source_settings.len()
    }
}

#[derive(Clone, Debug, Copy)]
pub enum ComposerType {
    Basic(BasicComposer),
    Status(StatusComposer),
}

#[derive(Clone, Debug, Copy)]
pub struct BasicComposer {
    pub data_sources: [Option<DataSource>; SOURCE_SIZE],
    pub ds_count: usize,
}

impl NodeModel for BasicComposer {
    type Input = ComposerSettings;
    fn to_input(&self) -> ComposerSettings {
        let ds: Vec<DataSource> = self.data_sources.into_iter().flatten().collect();
        let name: String = "basic".to_string();
        ComposerSettings {
            name,
            source_settings: ds,
        }
    }
}

impl BasicComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.ds_array(),
            ds_count: composer_settings.ds_count(),
        }
    }

    pub(crate) fn compose_sensor_data(&self, target_type: NodeType) -> SensorData {
        let mut sensor_data = SensorData::default();
        for idx in 0..self.ds_count {
            let ds_settings = self.data_sources[idx].unwrap();
            if ds_settings.target_type != target_type {
                continue;
            }
            let data_type = ds_settings.data_type;
            let data_counts = ds_settings.data_count;
            let data_size = ds_settings.unit_size * data_counts as f32;
            sensor_data.size_by_type.insert(data_type, data_size);
            sensor_data.count_by_type.insert(data_type, data_counts);
        }
        return sensor_data;
    }

    pub(crate) fn compose_payload(
        &self,
        target_type: NodeType,
        incoming_payload: Option<Vec<Payload>>,
    ) -> Payload {
        let mut payload = Payload::default();
        payload.sensor_data = self.compose_sensor_data(target_type);
        payload.total_size = payload.sensor_data.size_by_type.values().sum();
        payload.total_count = payload.sensor_data.count_by_type.values().sum();

        match incoming_payload {
            Some(mut payload_in) => {
                for single_payload in payload_in.iter_mut() {
                    payload
                        .downstream_data
                        .append(&mut single_payload.downstream_data);
                    payload.total_size += single_payload.total_size;
                    payload.total_count += single_payload.total_count;
                }
            }
            None => {}
        };
        return payload;
    }
}

#[derive(Clone, Debug, Copy)]
pub struct StatusComposer {
    data_sources: [Option<DataSource>; SOURCE_SIZE],
    ds_count: usize,
}

impl NodeModel for StatusComposer {
    type Input = ComposerSettings;
    fn to_input(&self) -> ComposerSettings {
        let ds: Vec<DataSource> = self.data_sources.into_iter().flatten().collect();
        let name: String = "basic".to_string();
        ComposerSettings {
            name,
            source_settings: ds,
        }
    }
}

impl StatusComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.ds_array(),
            ds_count: composer_settings.ds_count(),
        }
    }

    pub(crate) fn update_data_sources(&mut self, composer_settings: &ComposerSettings) {
        self.data_sources = composer_settings.ds_array();
        self.ds_count = composer_settings.ds_count();
    }

    pub(crate) fn compose_sensor_data(&self) -> SensorData {
        let mut sensor_data = SensorData::default();
        let data_size = 0.01;
        let data_count: u32 = 10;

        sensor_data.size_by_type.insert(DataType::Status, data_size);
        sensor_data
            .count_by_type
            .insert(DataType::Status, data_count);
        return sensor_data;
    }

    pub(crate) fn compose_payload(&self, incoming_payload: Option<Vec<Payload>>) -> Payload {
        let mut payload = Payload::default();
        payload.sensor_data = self.compose_sensor_data();
        payload.total_size = payload.sensor_data.size_by_type.values().sum();
        payload.total_count = payload.sensor_data.count_by_type.values().sum();

        match incoming_payload {
            Some(mut payload_in) => {
                for single_payload in payload_in.iter_mut() {
                    payload
                        .downstream_data
                        .append(&mut single_payload.downstream_data);
                    payload.total_size += single_payload.total_size;
                    payload.total_count += single_payload.total_count;
                }
            }
            None => {}
        };
        return payload;
    }
}
