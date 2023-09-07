use crate::data::data_io;
use crate::data::data_io::{DeviceId, TimeStamp, Trace};
use crate::data::stream_io;
use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::utils::config::{FieldSettings, PositionFiles, TraceFlags};
use crate::utils::constants::{
    BASE_STATION, COL_BASE_STATION_ID, COL_RSU_ID, COL_VEHICLE_ID, ROADSIDE_UNIT, VEHICLE,
};
use crate::DISCRETIZATION;
use itertools::izip;
use krabmaga::engine::fields::field::Field;
use krabmaga::engine::fields::field_2d::Field2D;
use krabmaga::engine::location::Real2D;
use krabmaga::hashbrown::HashMap;
use log::{debug, info};
use std::path::PathBuf;

pub(crate) type TraceMap = HashMap<TimeStamp, Option<Trace>>;

pub(crate) struct DeviceField {
    pub(crate) field_settings: FieldSettings,
    pub(crate) trace_flags: TraceFlags,
    pub(crate) vehicle_field: Field2D<Vehicle>,
    pub(crate) rsu_field: Field2D<RoadsideUnit>,
    pub(crate) bs_field: Field2D<BaseStation>,
    pub(crate) controller_field: Field2D<Controller>,
    pub(crate) vehicle_positions: TraceMap,
    pub(crate) rsu_positions: TraceMap,
    pub(crate) bs_positions: TraceMap,
    pub(crate) controller_positions: HashMap<DeviceId, (f32, f32)>,
    pub(crate) position_files: PositionFiles,
    pub(crate) config_path: PathBuf,
    pub(crate) position_cache: HashMap<DeviceId, Real2D>,
    pub(crate) velocity_cache: HashMap<DeviceId, f32>,
    pub(crate) streaming_interval: TimeStamp,
    pub(crate) step: TimeStamp,
}

impl DeviceField {
    pub(crate) fn new(
        field_settings: &FieldSettings,
        trace_flags: &TraceFlags,
        config_path: &PathBuf,
        position_files: &PositionFiles,
        streaming_interval: u64,
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
            trace_flags: trace_flags.clone(),
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
            streaming_interval,
            position_cache: Default::default(),
            velocity_cache: Default::default(),
        }
    }

    pub(crate) fn init(&mut self) {
        info! {"Initializing the device field"}
        self.vehicle_positions = self.read_vehicle_positions();
        self.rsu_positions = self.read_rsu_positions();
        self.bs_positions = self.read_base_station_positions();
        self.controller_positions = self.read_controller_positions();
    }

    pub(crate) fn clear(&mut self) {
        self.position_cache = HashMap::new();
        self.velocity_cache = HashMap::new();
    }

    pub(crate) fn before_step(&mut self, step: TimeStamp) {
        self.step = step;
        self.clear();
        self.refresh_vehicle_cache();
        self.refresh_rsu_cache();
        self.refresh_bs_cache();
    }

    pub(crate) fn update(&mut self) {
        self.vehicle_field.lazy_update();
        self.rsu_field.lazy_update();
        self.bs_field.lazy_update();
        self.controller_field.lazy_update();
    }

    fn refresh_vehicle_cache(&mut self) {
        if let Some(vehicle_traces) = self.vehicle_positions.get(&self.step) {
            let (vehicle_ids, xs, ys, velocities) = match vehicle_traces.as_ref() {
                Some(vehicle_traces) => vehicle_traces,
                None => {
                    debug! {"No vehicle traces found at step {}", self.step}
                    return;
                }
            };
            for (vehicle_id, x, y, velocity) in
                izip!(vehicle_ids.iter(), xs.iter(), ys.iter(), velocities.iter())
            {
                self.position_cache
                    .insert(*vehicle_id, Real2D { x: *x, y: *y });
                self.velocity_cache.insert(*vehicle_id, *velocity);
            }
        }
    }

    fn refresh_rsu_cache(&mut self) {
        if let Some(rsu_traces) = self.rsu_positions.get(&self.step) {
            let (rsu_ids, xs, ys, velocities) = match rsu_traces.as_ref() {
                Some(vehicle_traces) => vehicle_traces,
                None => {
                    debug! {"No RSU traces found at step {}", self.step}
                    return;
                }
            };
            for (rsu_id, x, y, _velocity) in
                izip!(rsu_ids.iter(), xs.iter(), ys.iter(), velocities.iter())
            {
                self.position_cache.insert(*rsu_id, Real2D { x: *x, y: *y });
                self.velocity_cache.insert(*rsu_id, 0.0);
            }
        }
    }

    fn refresh_bs_cache(&mut self) {
        if let Some(bs_positions) = self.bs_positions.get(&self.step) {
            let (bs_ids, xs, ys, velocities) = match bs_positions.as_ref() {
                Some(vehicle_traces) => vehicle_traces,
                None => {
                    debug! {"No base station traces found at step {}", self.step}
                    return;
                }
            };
            for (bs_id, x, y, velocity) in
                izip!(bs_ids.iter(), xs.iter(), ys.iter(), velocities.iter())
            {
                self.position_cache.insert(*bs_id, Real2D { x: *x, y: *y });
                self.velocity_cache.insert(*bs_id, 0.0);
            }
        }
    }

    pub(crate) fn refresh_position_data(&mut self, step: u64) {
        info! {"Refreshing position data from files at step {}", step}
        self.step = step;
        let vehicle_positions = self.read_vehicle_positions();

        if vehicle_positions.len() > 0 {
            self.vehicle_positions = vehicle_positions;
        }
    }

    fn stream_device_positions(&self, trace_file: PathBuf, device_id_column: &str) -> TraceMap {
        let starting_time: u64 = self.step;
        let ending_time: u64 = self.step + self.streaming_interval;
        let device_positions: TraceMap = stream_io::stream_positions_in_interval(
            trace_file,
            device_id_column,
            starting_time,
            ending_time,
        );
        return device_positions;
    }

    fn read_vehicle_positions(&self) -> TraceMap {
        let trace_file = self
            .config_path
            .join(&self.position_files.vehicle_positions);
        if trace_file.exists() == false {
            panic!("Vehicle trace file is not found.");
        }

        let vehicle_positions = if self.trace_flags.vehicle == true {
            self.stream_device_positions(trace_file, COL_VEHICLE_ID)
        } else {
            data_io::read_all_positions(trace_file, COL_VEHICLE_ID)
        };
        return vehicle_positions;
    }

    fn read_rsu_positions(&self) -> TraceMap {
        let trace_file = self.config_path.join(&self.position_files.rsu_positions);
        if trace_file.exists() == false {
            panic!("RSU trace file is not found.");
        }

        let rsu_positions = if self.trace_flags.roadside_unit == true {
            self.stream_device_positions(trace_file, COL_RSU_ID)
        } else {
            data_io::read_all_positions(trace_file, COL_RSU_ID)
        };
        return rsu_positions;
    }

    fn read_base_station_positions(&self) -> TraceMap {
        let trace_file = self.config_path.join(&self.position_files.bs_positions);
        if trace_file.exists() == false {
            panic!("Base station trace file is not found.");
        }

        let bs_positions = if self.trace_flags.base_station == true {
            self.stream_device_positions(trace_file, COL_BASE_STATION_ID)
        } else {
            data_io::read_all_positions(trace_file, COL_BASE_STATION_ID)
        };
        return bs_positions;
    }

    fn read_controller_positions(&self) -> HashMap<DeviceId, (f32, f32)> {
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
