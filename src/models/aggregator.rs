use crate::device::base_station::BSInfo;
use crate::models::composer::DevicePayload;
use krabmaga::hashbrown::HashMap;

#[derive(Clone, Debug, Copy)]
pub(crate) enum AggregatorType {
    Basic(BasicAggregator),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicAggregator;

#[derive(Clone, Debug, Default)]
pub(crate) struct BSPayload {
    pub(crate) id: u32,
    pub(crate) bs_info: BSInfo,
    pub(crate) v2bs_data: HashMap<u64, DevicePayload>,
    pub(crate) rsu2bs_data: HashMap<u64, DevicePayload>,
}

trait DataAggregator {
    fn aggregate_vehicle_data(&self);
}

impl DataAggregator for BasicAggregator {
    fn aggregate_vehicle_data(&self) {
        ()
    }
}
