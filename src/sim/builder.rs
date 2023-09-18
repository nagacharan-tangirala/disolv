use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::device_state::Timing;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::reader::activation;
use crate::reader::activation::{Activation, DeviceId, TimeStamp};
use crate::sim::core::Core;
use crate::sim::field::DeviceField;
use crate::sim::vanet::Vanet;
use crate::utils::config::VehicleSettings;
use crate::utils::config::{BaseStationSettings, ControllerSettings, RSUSettings};
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::{config, dyn_config, logger};
use krabmaga::hashbrown::HashMap;
use log::{debug, info};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct PavenetBuilder {
    base_config: config::BaseConfig,
    dyn_config: dyn_config::DynamicConfig,
    config_path: PathBuf,
}

impl PavenetBuilder {
    pub(crate) fn new(base_config_file: &str, dyn_config_file: &str) -> Self {
        if !Path::new(base_config_file).exists() {
            panic!("Configuration file is not found.");
        }
        let config_path = Path::new(base_config_file)
            .parent()
            .unwrap_or_else(|| {
                panic!("Invalid directory for the configuration file");
            })
            .to_path_buf();

        let dyn_config_reader = dyn_config::DynamicConfigReader::new(&dyn_config_file);
        let dyn_config = match dyn_config_reader.parse() {
            Ok(dyn_config) => dyn_config,
            Err(e) => {
                panic!("Error while parsing the dynamic configuration file: {}", e);
            }
        };

        let config_reader = config::BaseConfigReader::new(&base_config_file);
        match config_reader.parse() {
            Ok(base_config) => Self {
                base_config,
                dyn_config,
                config_path,
            },
            Err(e) => {
                panic!("Error while parsing the base configuration file: {}", e);
            }
        }
    }

    pub(crate) fn build(&mut self) -> Core {
        self.initiate_logger();
        let vehicles = self.build_vehicles();
        let roadside_units = self.build_roadside_units();
        let base_stations = self.build_base_stations();
        let controllers = self.build_controllers();

        debug! {"Building empty device field and VANET..."}
        let device_field: DeviceField = self.build_empty_device_field();
        let vanet: Vanet = self.build_empty_vanet();

        info!("Building the network...");
        return Core::new(
            self.base_config.clone(),
            self.dyn_config.clone(),
            vehicles,
            roadside_units,
            base_stations,
            controllers,
            device_field,
            vanet,
        );
    }

    pub(crate) fn initiate_logger(&self) {
        let log_level = &self.base_config.log_settings.log_level;
        let log_path = self
            .config_path
            .join(&self.base_config.log_settings.log_path);

        if log_path.exists() == false {
            fs::create_dir(&log_path)
                .unwrap_or_else(|_| panic!("Error while creating the log directory"));
        }

        let log_file_path = log_path.join(&self.base_config.log_settings.log_file_name);
        if log_file_path.exists() == true {
            // Clear the log file
            fs::remove_file(&log_file_path)
                .unwrap_or_else(|_| panic!("Error while clearing the log file"));
        }

        let logger_config = match logger::setup_logging(log_level, log_file_path) {
            Ok(logger_config) => logger_config,
            Err(e) => {
                panic!("Error while configuring the logger: {}", e);
            }
        };

        match log4rs::init_config(logger_config) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error while initializing logger with config: {}", e);
            }
        };
    }

    fn build_empty_device_field(&self) -> DeviceField {
        return DeviceField::new(
            &self.base_config.field_settings,
            &self.base_config.trace_flags,
            &self.config_path,
            &self.base_config.geo_data_files,
            self.base_config.simulation_settings.sim_streaming_step,
        );
    }

    fn build_empty_vanet(&self) -> Vanet {
        return Vanet::new(
            &self.config_path,
            &self.base_config.link_files,
            &self.base_config.network_settings,
            &self.base_config.trace_flags,
            self.base_config.simulation_settings.sim_streaming_step,
        );
    }

    fn build_vehicles(&mut self) -> HashMap<DeviceId, Vehicle> {
        info!("Building vehicles...");
        let activation_file = Path::new(&self.config_path)
            .join(&self.base_config.activation_files.vehicle_activations);
        if activation_file.exists() == false {
            panic!("Vehicle activation file is not found.");
        }
        let vehicle_activations: HashMap<DeviceId, Activation> =
            activation::read_activation_data(activation_file);

        let all_vehicle_settings: &Vec<VehicleSettings> = &self.base_config.vehicles;
        let ratios: Vec<f32> = all_vehicle_settings.iter().map(|vs| vs.ratio).collect();
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        let mut vehicles: HashMap<DeviceId, Vehicle> =
            HashMap::with_capacity(vehicle_activations.len());
        for (vehicle_id, activation_data) in vehicle_activations.iter() {
            let vehicle_setting = match all_vehicle_settings.get(dist.sample(&mut rng)) {
                Some(vehicle_setting) => vehicle_setting,
                None => {
                    panic!("Error while selecting vehicle settings.");
                }
            };

            let vehicle_timing = Self::convert_activation_to_timing(&activation_data);
            let new_vehicle = Vehicle::new(*vehicle_id, vehicle_timing, vehicle_setting);
            vehicles.entry(*vehicle_id).or_insert(new_vehicle);
        }
        info!("Done! Number of vehicles: {}", vehicles.len());
        return vehicles;
    }

    fn build_roadside_units(&self) -> HashMap<DeviceId, RoadsideUnit> {
        info!("Building roadside units...");
        let activation_file =
            Path::new(&self.config_path).join(&self.base_config.activation_files.rsu_activations);
        if activation_file.exists() == false {
            panic!("RSU activation file is not found.");
        }
        let rsu_activations: HashMap<DeviceId, Activation> =
            activation::read_activation_data(activation_file);

        let all_rsu_settings: &Vec<RSUSettings> = &self.base_config.roadside_units;
        let ratios: Vec<f32> = all_rsu_settings.iter().map(|rs| rs.ratio).collect();
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        let mut roadside_units: HashMap<DeviceId, RoadsideUnit> =
            HashMap::with_capacity(rsu_activations.len());
        for (rsu_id, activation_data) in rsu_activations.iter() {
            let rsu_settings = match all_rsu_settings.get(dist.sample(&mut rng)) {
                Some(rsu_setting) => rsu_setting,
                None => {
                    panic!("Error while selecting RSU settings.");
                }
            };

            let rsu_timing = Self::convert_activation_to_timing(&activation_data);
            let new_rsu = RoadsideUnit::new(*rsu_id, rsu_timing, rsu_settings);
            roadside_units.entry(*rsu_id).or_insert(new_rsu);
        }
        info!("Done! Number of Roadside Units: {}", roadside_units.len());
        return roadside_units;
    }

    fn build_base_stations(&self) -> HashMap<u64, BaseStation> {
        info!("Building base stations...");
        let activation_file = Path::new(&self.config_path)
            .join(&self.base_config.activation_files.base_station_activations);

        if activation_file.exists() == false {
            panic!("Base station activation file is not found.");
        }
        let bs_activations: HashMap<DeviceId, Activation> =
            activation::read_activation_data(activation_file);

        let mut base_stations: HashMap<u64, BaseStation> = HashMap::new();
        let all_bs_settings: &Vec<BaseStationSettings> = &self.base_config.base_stations;

        let ratios: Vec<f32> = all_bs_settings
            .iter()
            .map(|bs_settings| bs_settings.ratio)
            .collect();

        debug!("Base station ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (bs_id, activation_data) in bs_activations.iter() {
            let bs_timing = Self::convert_activation_to_timing(&activation_data);
            let bs_settings = match all_bs_settings.get(dist.sample(&mut rng)) {
                Some(bs_setting) => bs_setting,
                None => {
                    panic!("Error while selecting Base station settings.");
                }
            };
            let new_base_station = BaseStation::new(*bs_id, bs_timing, bs_settings);
            if let Some(value) = base_stations.insert(*bs_id, new_base_station) {
                panic!("Duplicate base station id: {}", value.id);
            }
        }
        info!("Done! Number of Base stations: {}", base_stations.len());
        return base_stations;
    }

    fn build_controllers(&self) -> HashMap<u64, Controller> {
        info!("Building controllers...");
        let activation_file = Path::new(&self.config_path)
            .join(&self.base_config.activation_files.controller_activations);

        if activation_file.exists() == false {
            panic!("Controller activation file is not found.");
        }

        let controller_activations: HashMap<DeviceId, Activation> =
            activation::read_activation_data(activation_file);

        let mut controllers: HashMap<u64, Controller> = HashMap::new();
        let all_controller_settings: &Vec<ControllerSettings> = &self.base_config.controllers;

        let ratios: Vec<f32> = all_controller_settings
            .iter()
            .map(|controller_settings| controller_settings.ratio)
            .collect();

        debug!("Controller ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (controller_id, activation_data) in controller_activations.iter() {
            let controller_timing = Self::convert_activation_to_timing(&activation_data);
            let controller_settings = match all_controller_settings.get(dist.sample(&mut rng)) {
                Some(controller_setting) => controller_setting,
                None => {
                    panic!("Error while selecting controller settings.");
                }
            };
            let base_controller =
                Controller::new(*controller_id, controller_timing, controller_settings);
            if let Some(value) = controllers.insert(*controller_id, base_controller) {
                panic!("Duplicate controller id: {}", value.id);
            }
        }
        info!("Done! Number of Controllers: {}", controllers.len());
        return controllers;
    }

    pub(crate) fn convert_activation_to_timing(activation: &Activation) -> Timing {
        let mut activation_times: [Option<u64>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        let mut deactivation_times: [Option<u64>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        for (i, start_time) in activation.0.iter().enumerate() {
            activation_times[i] = Some(*start_time);
        }
        for (i, end_time) in activation.1.iter().enumerate() {
            deactivation_times[i] = Some(*end_time);
            if i == 0 && end_time == &0 {
                // Device is always active
                deactivation_times[i] = Some(TimeStamp::MAX);
                break;
            }
        }
        Timing::new(activation_times, deactivation_times)
    }

    pub(crate) fn get_duration(&self) -> u64 {
        return self.base_config.simulation_settings.sim_duration;
    }
}
