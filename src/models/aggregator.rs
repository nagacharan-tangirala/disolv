use crate::device::base_station::BSPayload;
use crate::device::vehicle::VehiclePayload;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, SensorType};

#[derive(Clone, Debug, Copy)]
pub(crate) enum AggregatorType {
    Basic(BasicAggregator),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicAggregator;

trait DataAggregator {
    fn aggregate_vehicle_data(&self);
}

impl DataAggregator for BasicAggregator {
    fn aggregate_vehicle_data(&self) {
        ()
    }
}
