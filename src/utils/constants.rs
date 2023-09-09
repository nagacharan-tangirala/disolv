// Agents need Copy trait, which prevents vector usage.
// Hence, we have added arrays with fixed sizes.
// Make sure to change these values if relevant data size changes.
pub(crate) const ARRAY_SIZE: usize = 10;

// Dataframe columns
pub(crate) const COL_VEHICLE_ID: &str = "vehicle_id";
pub(crate) const COL_BASE_STATION_ID: &str = "base_station_id";
pub(crate) const COL_CONTROLLER_ID: &str = "controller_id";
pub(crate) const COL_RSU_ID: &str = "rsu_id";
pub(crate) const COL_DEVICE_ID: &str = "device_id";

pub(crate) const COL_VEHICLES: &str = "vehicles_str";
pub(crate) const COL_BASE_STATIONS: &str = "base_stations_str";
pub(crate) const COL_ROADSIDE_UNITS: &str = "roadside_units_str";
pub(crate) const COL_CONTROLLERS: &str = "controllers_str";

pub(crate) const COL_DISTANCES: &str = "distances_str";
pub(crate) const COL_VELOCITY: &str = "velocity";
pub(crate) const COL_START_TIME: &str = "start_time";
pub(crate) const COL_END_TIME: &str = "end_time";
pub(crate) const COL_TIME_STEP: &str = "time_step";

pub(crate) const COL_COORD_X: &str = "x";
pub(crate) const COL_COORD_Y: &str = "y";
