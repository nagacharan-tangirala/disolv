use crate::data::data_io::{PositionsReader, Trace};
use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::utils::config::PositionFiles;
use crate::utils::constants::{BASE_STATION_ID, RSU_ID, STREAM_TIME, VEHICLE_ID};
use crate::DISCRETIZATION;
use krabmaga::engine::fields::field::Field;
use krabmaga::engine::fields::field_2d::Field2D;
use krabmaga::hashbrown::HashMap;
use log::info;
use std::path::{Path, PathBuf};

pub(crate) struct DeviceField {
    pub(crate) vehicle_field: Field2D<Vehicle>,
    pub(crate) rsu_field: Field2D<RoadsideUnit>,
    pub(crate) bs_field: Field2D<BaseStation>,
    pub(crate) controller_field: Field2D<Controller>,
    pub(crate) vehicle_positions: HashMap<u64, Trace>,
    pub(crate) rsu_positions: HashMap<u64, Trace>,
    pub(crate) bs_positions: HashMap<u64, Trace>,
    pub(crate) controller_positions: HashMap<u64, (f32, f32)>,
    pub(crate) position_files: PositionFiles,
    pub(crate) config_path: PathBuf,
    pub(crate) position_reader: PositionsReader,
    pub(crate) step: u64,
}

impl DeviceField {
    pub(crate) fn new(
        x_max: f32,
        y_max: f32,
        config_path: &PathBuf,
        position_files: &PositionFiles,
    ) -> Self {
        let vehicle_positions = HashMap::new();
        let rsu_positions = HashMap::new();
        let bs_positions = HashMap::new();
        let controller_positions = HashMap::new();

        info!("Initializing individual device fields.");
        let vehicle_field = Field2D::new(x_max, y_max, DISCRETIZATION, false);
        let rsu_field = Field2D::new(x_max, y_max, DISCRETIZATION, false);
        let bs_field = Field2D::new(x_max, y_max, DISCRETIZATION, false);
        let controller_field = Field2D::new(x_max, y_max, DISCRETIZATION, false);

        info!("Initializing the combined device field.");
        DeviceField {
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
            position_reader: PositionsReader::new(),
            step: 0,
        }
    }

    pub(crate) fn init(&mut self) {
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
        self.step = step;
        let vehicle_positions = self.read_vehicle_positions();
        let rsu_positions = self.read_rsu_positions();
        let bs_positions = self.read_base_station_positions();

        if vehicle_positions.len() > 0 {
            self.vehicle_positions = vehicle_positions;
        }
        if rsu_positions.len() > 0 {
            self.rsu_positions = rsu_positions;
        }
        if bs_positions.len() > 0 {
            self.bs_positions = bs_positions;
        }
    }

    fn read_vehicle_positions(&self) -> HashMap<u64, Trace> {
        let trace_file = self
            .config_path
            .join(&self.position_files.vehicle_positions);
        if trace_file.exists() == false {
            panic!("Vehicle trace file is not found.");
        }

        let starting_time: u64 = self.step;
        let ending_time: u64 = (self.step + STREAM_TIME);
        let vehicle_positions: HashMap<u64, Trace> = self.position_reader.read_position_data(
            trace_file,
            VEHICLE_ID,
            starting_time,
            ending_time,
        );
        vehicle_positions
    }

    fn read_rsu_positions(&self) -> HashMap<u64, Trace> {
        let trace_file = self.config_path.join(&self.position_files.rsu_positions);
        if trace_file.exists() == false {
            panic!("RSU trace file is not found.");
        }

        let starting_time: u64 = self.step;
        let ending_time: u64 = self.step + STREAM_TIME;
        let rsu_positions: HashMap<u64, Trace> =
            self.position_reader
                .read_position_data(trace_file, RSU_ID, starting_time, ending_time);
        rsu_positions
    }

    fn read_base_station_positions(&self) -> HashMap<u64, Trace> {
        let trace_file = self.config_path.join(&self.position_files.bs_positions);
        if trace_file.exists() == false {
            panic!("Base station trace file is not found.");
        }

        let starting_time: u64 = self.step;
        let ending_time: u64 = self.step + STREAM_TIME;
        let bs_positions: HashMap<u64, Trace> = self.position_reader.read_position_data(
            trace_file,
            BASE_STATION_ID,
            starting_time,
            ending_time,
        );
        bs_positions
    }

    fn read_controller_positions(&self) -> HashMap<u64, (f32, f32)> {
        let trace_file = self
            .config_path
            .join(&self.position_files.controller_positions);
        if trace_file.exists() == false {
            panic!("Controller trace file is not found.");
        }

        let controller_positions: HashMap<u64, (f32, f32)> =
            match self.position_reader.read_controller_positions(trace_file) {
                Ok(controller_positions) => controller_positions,
                Err(e) => {
                    panic!("Error while reading controller positions: {}", e);
                }
            };
        controller_positions
    }
}
