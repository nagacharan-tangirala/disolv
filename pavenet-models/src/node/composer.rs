use pavenet_config::config::base::{ComposerSettings, DataType, SourceSettings};
use pavenet_config::types::ids::node::NodeId;

pub const SOURCE_SIZE: usize = 10;

#[derive(Clone, Debug, Copy)]
pub enum ComposerType {
    Basic(BasicComposer),
    Status(StatusComposer),
}

#[derive(Clone, Debug, Copy)]
pub struct BasicComposer {
    pub data_sources: [Option<SourceSettings>; SOURCE_SIZE],
    pub ds_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Payload {
    pub sensor_data: SensorData,
    pub total_size: f32,
    pub total_count: u32,
    pub downstream_data: Vec<NodeId>,
}

#[derive(Clone, Debug, Default)]
pub struct SensorData {
    pub device_info: NodeInfo,
    pub map_state: MapState,
    pub size_by_type: HashMap<DataType, f32>,
}

#[derive(Clone, Debug, Copy)]
pub struct StatusComposer {
    data_sources: [Option<SourceSettings>; SOURCE_SIZE],
    ds_count: usize,
}

impl BasicComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        let ds_settings = &composer_settings.source_settings;
        let mut data_sources = [None; SOURCE_SIZE];
        for idx in 0..ds_settings.len() {
            data_sources[idx] = Some(composer_settings.source_settings[idx]);
        }
        Self {
            data_sources,
            ds_count: ds_settings.len(),
        }
    }

    pub(crate) fn update_data_sources(&mut self, composer_settings: &ComposerSettings) {
        let ds_settings = &composer_settings.source_settings;
        self.data_sources = [None; SOURCE_SIZE];
        for idx in 0..ds_settings.len() {
            self.data_sources[idx] = Some(ds_settings[idx]);
        }
        self.ds_count = ds_settings.len();
    }

    pub(crate) fn compose_sensor_data(&self) -> SensorData {
        let mut sensor_data = SensorData::default();
        sensor_data.temperature = 25.0;
        sensor_data.env_temperature = 32.0;
        for idx in 0..self.ds_count {
            // Unwrap is safe here because type counts is the length of the array.
            let ds_settings = self.data_sources[idx].unwrap();
            let data_type = ds_settings.data_type;
            let data_counts = ds_settings.data_count;
            let data_size = ds_settings.unit_size * data_counts as f32;

            sensor_data.size_by_type.insert(data_type, data_size);
            sensor_data.count_by_type.insert(data_type, data_counts);
        }
        return sensor_data;
    }

    pub(crate) fn compose_payload(&self, incoming_payload: Option<Vec<Payload>>) -> Payload {
        let mut payload = Payload::default();
        payload.sensor_data = self.compose_sensor_data();
        payload.total_size = payload.sensor_data.size_by_type.values().sum();
        payload.total_count = payload.sensor_data.size_by_type.values().sum();

        match incoming_payload {
            Some(incoming_payload) => {
                payload
                    .downstream_data
                    .append(incoming_payload.iter().map(|x| x.sensor_data).collect());
                payload.total_size += incoming_payload.iter().map(|x| x.total_size).sum();
                payload.total_count += incoming_payload.iter().map(|x| x.total_count).sum();
            }
            None => {}
        };
        return payload;
    }
}

impl StatusComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        let ds_settings = &composer_settings.source_settings;
        let mut data_sources = [None; SOURCE_SIZE];
        for idx in 0..ds_settings.len() {
            data_sources[idx] = Some(composer_settings.source_settings[idx]);
        }
        let ds_count = ds_settings.len();
        Self {
            data_sources,
            ds_count,
        }
    }

    pub(crate) fn update_data_sources(&mut self, composer_settings: &ComposerSettings) {
        let ds_settings = &composer_settings.source_settings;
        self.data_sources = [None; SOURCE_SIZE];
        for idx in 0..ds_settings.len() {
            self.data_sources[idx] = Some(ds_settings[idx]);
        }
        self.ds_count = ds_settings.len();
    }

    pub(crate) fn compose_sensor_data(&self) -> SensorData {
        let mut sensor_data = SensorData::default();
        sensor_data.temperature = 25.0;
        sensor_data.env_temperature = 32.0;

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
        payload.total_count = payload.sensor_data.size_by_type.values().sum();

        match incoming_payload {
            Some(incoming_payload) => {
                payload
                    .downstream_data
                    .append(incoming_payload.iter().map(|x| x.sensor_data).collect());
                payload.total_size += incoming_payload.iter().map(|x| x.total_size).sum();
                payload.total_count += incoming_payload.iter().map(|x| x.total_count).sum();
            }
            None => {}
        };
        return payload;
    }
}
