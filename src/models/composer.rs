use crate::device::vehicle::VehiclePayload;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, SensorType};

#[derive(Clone, Debug, Copy)]
pub(crate) enum ComposerType {
    Basic(BasicComposer),
    Random(RandomComposer),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicComposer {
    pub(crate) data_sources: [Option<DataSourceSettings>; ARRAY_SIZE],
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct RandomComposer {
    pub(crate) data_sources: [Option<DataSourceSettings>; ARRAY_SIZE],
}

trait DataComposer {
    fn compose_payload(&self) -> VehiclePayload;
}

impl DataComposer for BasicComposer {
    fn compose_payload(&self) -> VehiclePayload {
        let mut payload = VehiclePayload::default();
        for i in 0..self.data_sources.len() {
            let data_source: DataSourceSettings = match self.data_sources[i] {
                Some(ds) => ds,
                None => continue,
            };

            let data_type = data_source.data_type;
            let data_counts: u16 = data_source.data_counts;
            let data_size = data_source.unit_size * data_source.data_counts as f32;
            payload.generated_data_size.insert(data_type, data_size);
            payload.types_with_counts.insert(data_type, data_counts);
            payload
                .preferred_targets
                .insert(data_type, data_source.target_type);
        }
        return payload;
    }
}

impl DataComposer for RandomComposer {
    fn compose_payload(&self) -> VehiclePayload {
        let payload = VehiclePayload::default();
        return payload;
    }
}
