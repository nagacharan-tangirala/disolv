use crate::data::data_io;
use crate::data::data_io::Trace;
use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::utils::config::{DeviceFieldSettings, PositionFiles};
use crate::utils::constants::{
    BASE_STATION, COL_BASE_STATION_ID, COL_RSU_ID, COL_VEHICLE_ID, ROADSIDE_UNIT, STREAM_TIME,
    VEHICLE,
};
use crate::DISCRETIZATION;
use krabmaga::engine::fields::field::Field;
use krabmaga::engine::fields::field_2d::Field2D;
use krabmaga::hashbrown::HashMap;
use log::info;
use std::path::PathBuf;

pub(crate) struct DeviceField {
    pub(crate) field_settings: DeviceFieldSettings,
    pub(crate) vehicle_field: Field2D<Vehicle>,
    pub(crate) rsu_field: Field2D<RoadsideUnit>,
    pub(crate) bs_field: Field2D<BaseStation>,
    pub(crate) controller_field: Field2D<Controller>,
    pub(crate) vehicle_positions: HashMap<u64, Option<Trace>>,
    pub(crate) rsu_positions: HashMap<u64, Option<Trace>>,
    pub(crate) bs_positions: HashMap<u64, Option<Trace>>,
    pub(crate) controller_positions: HashMap<u64, (f32, f32)>,
    pub(crate) position_files: PositionFiles,
    pub(crate) config_path: PathBuf,
    pub(crate) step: u64,
}

impl DeviceField {
    pub(crate) fn new(
        field_settings: &DeviceFieldSettings,
        config_path: &PathBuf,
        position_files: &PositionFiles,
    ) -> Self {
        let vehicle_positions = HashMap::new();
        let rsu_positions = HashMap::new();
        let bs_positions = HashMap::new();
        let controller_positions = HashMap::new();

        let vehicle_field = Field2D::new(
            field_settings.width,
            field_settings.height,
            DISCRETIZATION,
            false,
        );
        let rsu_field = Field2D::new(
            field_settings.width,
            field_settings.height,
            DISCRETIZATION,
            false,
        );
        let bs_field = Field2D::new(
            field_settings.width,
            field_settings.height,
            DISCRETIZATION,
            false,
        );
        let controller_field = Field2D::new(
            field_settings.width,
            field_settings.height,
            DISCRETIZATION,
            false,
        );

        Self {
            field_settings: field_settings.clone(),
            vehicle_field,
            rsu_field,
            bs_field,
            controller_field,
            vehicle_positions,
            rsu_positions,
            bs_positions,
            controller_positions,
            position_files: position_files.clone(),
            config_path: config_path.clone(),
            step: 0,
        }
    }

    pub(crate) fn init(&mut self) {
        info! {"Initializing the device field"}
        self.vehicle_positions = self.read_vehicle_positions();
        self.rsu_positions = self.read_rsu_positions();
        self.bs_positions = self.read_base_station_positions();
        self.controller_positions = self.read_controller_positions();
    }

    pub(crate) fn update(&mut self) {
        self.vehicle_field.lazy_update();
        self.rsu_field.lazy_update();
        self.bs_field.lazy_update();
        self.controller_field.lazy_update();
    }

    pub(crate) fn refresh_position_data(&mut self, step: u64) {
        info! {"Refreshing position data from files at step {}", step}
        self.step = step;
        let vehicle_positions = self.read_vehicle_positions();

        if vehicle_positions.len() > 0 {
            self.vehicle_positions = vehicle_positions;
        }
    }

    fn stream_device_positions(
        &self,
        trace_file: PathBuf,
        device_id_column: &str,
    ) -> HashMap<u64, Option<Trace>> {
        let starting_time: u64 = self.step;
        let ending_time: u64 = self.step + STREAM_TIME;
        let vehicle_positions: HashMap<u64, Option<Trace>> = data_io::stream_positions_in_interval(
            trace_file,
            device_id_column,
            starting_time,
            ending_time,
        );
        return vehicle_positions;
    }

    fn read_all_positions(
        &self,
        trace_file: PathBuf,
        device_id_column: &str,
    ) -> HashMap<u64, Option<Trace>> {
        let device_positions: HashMap<u64, Option<Trace>> =
            data_io::read_all_positions(trace_file, device_id_column);
        device_positions
    }

    fn read_vehicle_positions(&self) -> HashMap<u64, Option<Trace>> {
        let trace_file = self
            .config_path
            .join(&self.position_files.vehicle_positions);
        if trace_file.exists() == false {
            panic!("Vehicle trace file is not found.");
        }

        let trace_flag: bool = match self.field_settings.trace_flags.get(VEHICLE) {
            Some(trace_flag) => *trace_flag,
            None => {
                panic!("RSU trace flag is not set in the config file.")
            }
        };

        let vehicle_positions = if trace_flag == true {
            self.stream_device_positions(trace_file, COL_VEHICLE_ID)
        } else {
            self.read_all_positions(trace_file, COL_VEHICLE_ID)
        };
        return vehicle_positions;
    }

    fn read_rsu_positions(&self) -> HashMap<u64, Option<Trace>> {
        let trace_file = self.config_path.join(&self.position_files.rsu_positions);
        if trace_file.exists() == false {
            panic!("RSU trace file is not found.");
        }

        let trace_flag: bool = match self.field_settings.trace_flags.get(ROADSIDE_UNIT) {
            Some(trace_flag) => *trace_flag,
            None => {
                panic!("RSU trace flag is not set in the config file.")
            }
        };

        let rsu_positions = if trace_flag == true {
            self.stream_device_positions(trace_file, COL_RSU_ID)
        } else {
            self.read_all_positions(trace_file, COL_RSU_ID)
        };
        return rsu_positions;
    }

    fn read_base_station_positions(&self) -> HashMap<u64, Option<Trace>> {
        let trace_file = self.config_path.join(&self.position_files.bs_positions);
        if trace_file.exists() == false {
            panic!("Base station trace file is not found.");
        }

        let trace_flag: bool = match self.field_settings.trace_flags.get(BASE_STATION) {
            Some(trace_flag) => *trace_flag,
            None => {
                panic!("Base station trace flag is not set in the config file.")
            }
        };

        let bs_positions = if trace_flag == true {
            self.stream_device_positions(trace_file, COL_BASE_STATION_ID)
        } else {
            self.read_all_positions(trace_file, COL_BASE_STATION_ID)
        };
        return bs_positions;
    }

    fn read_controller_positions(&self) -> HashMap<u64, (f32, f32)> {
        let trace_file = self
            .config_path
            .join(&self.position_files.controller_positions);
        if trace_file.exists() == false {
            panic!("Controller trace file is not found.");
        }

        let controller_positions: HashMap<u64, (f32, f32)> =
            match data_io::read_controller_positions(trace_file) {
                Ok(controller_positions) => controller_positions,
                Err(e) => {
                    panic!("Error while reading controller positions: {}", e);
                }
            };
        controller_positions
    }
}
