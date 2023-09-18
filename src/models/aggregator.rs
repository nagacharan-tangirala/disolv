use crate::device::base_station::BSInfo;
use crate::models::composer::DevicePayload;
use crate::reader::activation::DeviceId;
use crate::utils::ds_config::SensorType;
use krabmaga::hashbrown::HashMap;

#[derive(Clone, Debug, Copy)]
pub(crate) enum AggregatorType {
    Basic(BasicAggregator),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicAggregator;

#[derive(Clone, Debug, Default)]
pub(crate) struct InfraPayload {
    pub(crate) id: DeviceId,
    pub(crate) bs_info: BSInfo,
    pub(crate) vehicle_count: usize,
    pub(crate) rsu_count: usize,
    pub(crate) vehicle_ids: Vec<DeviceId>,
    pub(crate) rsu_ids: Vec<DeviceId>,
    pub(crate) vehicle_data_size: HashMap<DataType, f32>,
    pub(crate) rsu_data_size: HashMap<DataType, f32>,
    pub(crate) vehicle_data_counts: HashMap<DataType, u32>,
    pub(crate) rsu_data_counts: HashMap<DataType, u32>,
}

impl BasicAggregator {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) fn aggregate(
        &self,
        v2bs_data: Vec<DevicePayload>,
        rsu2bs_data: Vec<DevicePayload>,
    ) -> InfraPayload {
        let mut bs_payload = InfraPayload::default();
        bs_payload.vehicle_count = v2bs_data.len();
        bs_payload.rsu_count = rsu2bs_data.len();

        for payload in v2bs_data.iter() {
            bs_payload.vehicle_ids.push(payload.id);
            for (sensor_type, data_size) in payload.generated_data_size.iter() {
                bs_payload
                    .vehicle_data_size
                    .entry(*sensor_type)
                    .and_modify(|e| *e += *data_size)
                    .or_insert(*data_size);
            }
            for (sensor_type, data_count) in payload.types_with_counts.iter() {
                bs_payload
                    .vehicle_data_counts
                    .entry(*sensor_type)
                    .and_modify(|e| *e += *data_count)
                    .or_insert(*data_count);
            }
        }
        for payload in rsu2bs_data.iter() {
            bs_payload.rsu_ids.push(payload.id);
            for (sensor_type, data_size) in payload.generated_data_size.iter() {
                bs_payload
                    .rsu_data_size
                    .entry(*sensor_type)
                    .and_modify(|e| *e += *data_size)
                    .or_insert(*data_size);
            }
            for (sensor_type, data_count) in payload.types_with_counts.iter() {
                bs_payload
                    .rsu_data_counts
                    .entry(*sensor_type)
                    .and_modify(|e| *e += *data_count)
                    .or_insert(*data_count);
            }
        }
        return bs_payload;
    }
}
