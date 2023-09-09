use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::reader::activation::{DeviceId, TimeStamp};
use crate::reader::geo::GeoReader;
use crate::utils::config::{FieldSettings, GeoDataFiles, TraceFlags};
use crate::DISCRETIZATION;
use itertools::izip;
use krabmaga::engine::fields::field::Field;
use krabmaga::engine::fields::field_2d::Field2D;
use krabmaga::engine::location::Real2D;
use krabmaga::hashbrown::HashMap;
use log::{debug, info};
use std::path::PathBuf;

pub(crate) type GeoData = (Vec<DeviceId>, Vec<f32>, Vec<f32>, Vec<f32>); // (device_id, x, y, velocity)
pub(crate) type GeoMap = HashMap<TimeStamp, Option<GeoData>>;

pub(crate) struct DeviceField {
    pub(crate) field_settings: FieldSettings,
    pub(crate) vehicle_field: Field2D<Vehicle>,
    pub(crate) rsu_field: Field2D<RoadsideUnit>,
    pub(crate) bs_field: Field2D<BaseStation>,
    pub(crate) controller_field: Field2D<Controller>,
    pub(crate) vehicle_geo_data: GeoMap,
    pub(crate) rsu_geo_data: GeoMap,
    pub(crate) bs_geo_data: GeoMap,
    pub(crate) controller_positions: HashMap<DeviceId, (f32, f32)>,
    pub(crate) geo_reader: GeoReader,
    pub(crate) position_cache: HashMap<DeviceId, Real2D>,
    pub(crate) velocity_cache: HashMap<DeviceId, f32>,
    pub(crate) step: TimeStamp,
}

impl DeviceField {
    pub(crate) fn new(
        field_settings: &FieldSettings,
        trace_flags: &TraceFlags,
        config_path: &PathBuf,
        position_files: &GeoDataFiles,
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

        let geo_reader =
            GeoReader::new(config_path, position_files, trace_flags, streaming_interval);

        Self {
            field_settings: field_settings.clone(),
            vehicle_field,
            rsu_field,
            bs_field,
            controller_field,
            vehicle_geo_data: vehicle_positions,
            rsu_geo_data: rsu_positions,
            bs_geo_data: bs_positions,
            controller_positions,
            step: 0,
            position_cache: Default::default(),
            velocity_cache: Default::default(),
            geo_reader,
        }
    }

    pub(crate) fn init(&mut self) {
        info! {"Initializing the device field"}
        self.vehicle_geo_data = self.geo_reader.read_vehicle_geo_data();
        self.rsu_geo_data = self.geo_reader.read_rsu_geo_data();
        self.bs_geo_data = self.geo_reader.read_bs_geo_data();
        self.controller_positions = self.geo_reader.read_controller_geo_data();
    }

    pub(crate) fn before_step(&mut self, step: TimeStamp) {
        self.step = step;
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

    pub(crate) fn refresh_position_data(&mut self, step: TimeStamp) {
        info! {"Refreshing position data from files at step {}", step}
        self.geo_reader.step = step;
        let vehicle_positions = self.geo_reader.read_vehicle_geo_data();

        if vehicle_positions.len() > 0 {
            self.vehicle_geo_data = vehicle_positions;
        }
    }

    fn refresh_vehicle_cache(&mut self) {
        if let Some(vehicle_traces) = self.vehicle_geo_data.remove(&self.step) {
            let (vehicle_ids, xs, ys, velocities) = match vehicle_traces {
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
        if let Some(rsu_traces) = self.rsu_geo_data.remove(&self.step) {
            let (rsu_ids, xs, ys, velocities) = match rsu_traces {
                Some(rsu_traces) => rsu_traces,
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
        if let Some(bs_positions) = self.bs_geo_data.remove(&self.step) {
            let (bs_ids, xs, ys, velocities) = match bs_positions {
                Some(bs_traces) => bs_traces,
                None => {
                    debug! {"No base station traces found at step {}", self.step}
                    return;
                }
            };
            for (bs_id, x, y, _velocity) in
                izip!(bs_ids.iter(), xs.iter(), ys.iter(), velocities.iter())
            {
                self.position_cache.insert(*bs_id, Real2D { x: *x, y: *y });
                self.velocity_cache.insert(*bs_id, 0.0);
            }
        }
    }
}
