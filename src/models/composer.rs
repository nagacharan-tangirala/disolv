use crate::reader::activation::DeviceId;
use crate::utils::config::DeviceType;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::dyn_config::DataType;
use krabmaga::hashbrown::HashMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct UplinkPayload {
    pub(crate) id: DeviceId,
    pub(crate) sensor_data: SensorData,
    pub(crate) generated_data_size: HashMap<DataType, f32>,
    pub(crate) types_with_counts: HashMap<DataType, u32>,
    pub(crate) total_data_size: f32,
    pub(crate) total_data_count: u32,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct SensorData {
    pub(crate) speed: f32,
    pub(crate) temperature: f32,
    pub(crate) env_temperature: f32,
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct DataSources {
    pub(crate) data_type: DataType,
    pub(crate) unit_size: f32,
    pub(crate) data_counts: u32,
}

#[derive(Clone, Debug, Copy)]
pub(crate) enum ComposerType {
    Basic(BasicComposer),
    Random(RandomComposer),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicComposer {
    pub(crate) data_sources: [Option<DataSources>; ARRAY_SIZE],
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct RandomComposer {
    pub(crate) data_sources: [Option<DataSources>; ARRAY_SIZE],
}

impl BasicComposer {
    pub(crate) fn new(data_sources: [Option<DataSources>; ARRAY_SIZE]) -> Self {
        Self { data_sources }
    }

    pub(crate) fn compose_payload(&self, _target_type: DeviceType) -> UplinkPayload {
        let mut payload = UplinkPayload::default();
        let total_data_size = 0.0;
        let total_data_count: u32 = 0;
        for data_source in self.data_sources {
            let data_source: DataSources = match data_source {
                Some(ds) => ds,
                None => continue,
            };

            let data_type = data_source.data_type;
            let data_counts: u32 = data_source.data_counts;
            let data_size = data_source.unit_size * data_source.data_counts as f32;

            payload.generated_data_size.insert(data_type, data_size);
            payload.types_with_counts.insert(data_type, data_counts);
        }
        payload.total_data_count = total_data_count;
        payload.total_data_size = total_data_size;
        return payload;
    }
}

impl RandomComposer {
    pub(crate) fn new(data_sources: [Option<DataSources>; ARRAY_SIZE]) -> Self {
        Self { data_sources }
    }

    pub(crate) fn compose_payload(&self) -> UplinkPayload {
        let payload = UplinkPayload::default();
        return payload;
    }
}
