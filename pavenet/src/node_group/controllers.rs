use pavenet_config::config::types::{DeviceId, TimeStamp};
use pavenet_core::::controller::Controller;
use crate::devices::controller::Controller;

pub struct Controllers {
    controllers: Vec<Controller>,
    pub devices_to_add: Vec<(DeviceId, TimeStamp)>,
    pub devices_to_pop: Vec<DeviceId>,
}
