use crate::devices::bs::BaseStation;
use pavenet_config::config::types::{DeviceId, TimeStamp};

pub struct BaseStations {
    base_stations: Vec<BaseStation>,
    pub devices_to_add: Vec<(DeviceId, TimeStamp)>,
    pub devices_to_pop: Vec<DeviceId>,
}
