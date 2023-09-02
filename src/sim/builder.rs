use crate::data::data_io::{Activation, ActivationDataReader};
use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::sim::field::DeviceField;
use crate::sim::network::{Network, Timing};
use crate::sim::vanet::{InfraLinks, MeshLinks, Vanet};
use crate::utils::config::{BaseStationSettings, RSUSettings, VehicleSettings};
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::AllDataSources;
use crate::utils::{config, ds_config, logger};
use krabmaga::hashbrown::HashMap;
use log::info;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct PavenetBuilder {
    config: config::Config,
    ds_config: AllDataSources,
    config_path: PathBuf,
    activation_data_reader: ActivationDataReader,
}

impl PavenetBuilder {
    pub(crate) fn new(config_file: &str) -> Self {
        if !Path::new(config_file).exists() {
            panic!("Configuration file is not found.");
        }
        let config_path = Path::new(config_file)
            .parent()
            .unwrap_or_else(|| {
                panic!("Invalid directory for the configuration file");
            })
            .to_path_buf();

        let config_reader = config::ConfigReader::new(&config_file);
        let activation_data_reader = ActivationDataReader::new();
        let config_data = match config_reader.parse() {
            Ok(config_data) => config_data,
            Err(e) => {
                panic!("Error while parsing the configuration file: {}", e);
            }
        };

        let data_source_file =
            Path::new(&config_path).join(&config_data.data_source_config_file.config_file);
        let ds_config_reader = ds_config::DSConfigReader::new(&data_source_file);
        match ds_config_reader.parse() {
            Ok(ds_config) => Self {
                config: config_data,
                ds_config,
                config_path,
                activation_data_reader,
            },
            Err(e) => {
                panic!("Error while parsing the data source config file: {}", e);
            }
        }
    }

    pub(crate) fn build(&mut self) -> Network {
        self.initiate_logger();
        let vehicles = self.build_vehicles();
        let roadside_units = self.build_roadside_units();
        let base_stations = self.build_base_stations();
        let controllers = self.build_controllers();

        info! {"Building empty device field and VANET..."};
        let device_field = self.build_empty_device_field();
        let vanet: Vanet = self.build_empty_vanet();

        info!("Building the network...");
        return Network::new(
            self.config.clone(),
            self.ds_config.clone(),
            vehicles,
            roadside_units,
            base_stations,
            controllers,
            device_field,
            vanet,
        );
    }

    pub(crate) fn initiate_logger(&self) {
        let log_level = &self.config.log_settings.log_level;
        let log_path = self.config_path.join(&self.config.log_settings.log_path);

        if log_path.exists() == false {
            fs::create_dir(&log_path)
                .unwrap_or_else(|_| panic!("Error while creating the log directory"));
        }

        let log_file_path = log_path.join(&self.config.log_settings.log_file_name);

        let logger_config = match logger::setup_logging(log_level, log_file_path) {
            Ok(logger_config) => logger_config,
            Err(e) => {
                panic!("Error while configuring the loggerr: {}", e);
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
        let x_max = self.config.simulation_settings.dimension_x_max;
        let y_max = self.config.simulation_settings.dimension_y_max;
        return DeviceField::new(x_max, y_max, &self.config_path, &self.config.position_files);
    }

    fn build_empty_vanet(&self) -> Vanet {
        let mesh_links = MeshLinks::new();
        let infra_links = InfraLinks::new();
        return Vanet::new(mesh_links, infra_links);
    }

    fn build_vehicles(&mut self) -> HashMap<u64, Vehicle> {
        info!("Building vehicles...");
        let activation_file =
            Path::new(&self.config_path).join(&self.config.activation_files.vehicle_activations);
        if activation_file.exists() == false {
            panic!("Vehicle activation file is not found.");
        }
        let vehicle_activations: HashMap<u64, Activation> = self
            .activation_data_reader
            .read_activation_data(activation_file);

        let mut vehicles: HashMap<u64, Vehicle> = HashMap::new();
        let all_vehicle_settings: Vec<&VehicleSettings> = self.config.vehicles.values().collect();
        let ratios: Vec<f32> = all_vehicle_settings
            .iter()
            .map(|vehicle_setting| vehicle_setting.ratio)
            .collect();

        info!("Vehicle ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (vehicle_id, activation_data) in vehicle_activations.iter() {
            let vehicle_setting = match all_vehicle_settings.get(dist.sample(&mut rng)) {
                Some(vehicle_setting) => *vehicle_setting,
                None => {
                    panic!("Error while selecting vehicle settings.");
                }
            };
            let vehicle_timing = Self::convert_activation_to_timing(&activation_data);
            let new_vehicle = Vehicle::new(*vehicle_id, vehicle_timing, vehicle_setting);
            if let Some(value) = vehicles.insert(*vehicle_id, new_vehicle) {
                panic!("Duplicate Vehicle id: {}", value.id);
            }
        }
        info!("Number of vehicles: {}", vehicles.len());
        return vehicles;
    }

    fn build_roadside_units(&self) -> HashMap<u64, RoadsideUnit> {
        info!("Building roadside units...");
        let activation_file =
            Path::new(&self.config_path).join(&self.config.activation_files.rsu_activations);
        if activation_file.exists() == false {
            panic!("RSU activation file is not found.");
        }
        let rsu_activations: HashMap<u64, data_io::Activation> =
            data_io::read_activation_data(activation_file);

        let mut roadside_units: HashMap<u64, RoadsideUnit> = HashMap::new();
        let all_rsu_settings: Vec<&RSUSettings> = self.config.roadside_units.values().collect();
        let ratios: Vec<f32> = all_rsu_settings
            .iter()
            .map(|rsu_settings| rsu_settings.ratio)
            .collect();

        info!("RSU ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (rsu_id, activation_data) in rsu_activations.iter() {
            let rsu_settings = match all_rsu_settings.get(dist.sample(&mut rng)) {
                Some(rsu_setting) => *rsu_setting,
                None => {
                    panic!("Error while selecting RSU settings.");
                }
            };
            let rsu_timing = Self::convert_activation_to_timing(&activation_data);
            let new_rsu = RoadsideUnit::new(*rsu_id, rsu_timing, rsu_settings);
            if let Some(value) = roadside_units.insert(*rsu_id, new_rsu) {
                panic!("Duplicate RSU id: {}", value.id);
            }
        }
        info!("Number of Roadside Units: {}", roadside_units.len());
        return roadside_units;
    }

    fn build_base_stations(&self) -> HashMap<u64, BaseStation> {
        info!("Building base stations...");
        let activation_file = Path::new(&self.config_path)
            .join(&self.config.activation_files.base_station_activations);

        if activation_file.exists() == false {
            panic!("Base station activation file is not found.");
        }
        let bs_activations: HashMap<u64, data_io::Activation> =
            data_io::read_activation_data(activation_file);

        let mut base_stations: HashMap<u64, BaseStation> = HashMap::new();
        let all_bs_settings: Vec<&BaseStationSettings> =
            self.config.base_stations.values().collect();

        let ratios: Vec<f32> = all_bs_settings
            .iter()
            .map(|bs_settings| bs_settings.ratio)
            .collect();

        info!("Base station ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (bs_id, activation_data) in bs_activations.iter() {
            let bs_timing = Self::convert_activation_to_timing(&activation_data);
            let bs_settings = match all_bs_settings.get(dist.sample(&mut rng)) {
                Some(bs_setting) => *bs_setting,
                None => {
                    panic!("Error while selecting Base station settings.");
                }
            };
            let new_base_station = BaseStation::new(*bs_id, bs_timing, bs_settings);
            if let Some(value) = base_stations.insert(*bs_id, new_base_station) {
                panic!("Duplicate base station id: {}", value.id);
            }
        }
        info!("Number of Base stations: {}", base_stations.len());
        return base_stations;
    }

    fn build_controllers(&self) -> HashMap<u64, Controller> {
        info!("Building controllers...");
        let activation_file =
            Path::new(&self.config_path).join(&self.config.activation_files.controller_activations);

        if activation_file.exists() == false {
            panic!("Controller activation file is not found.");
        }

        let controller_activations: HashMap<u64, data_io::Activation> =
            data_io::read_activation_data(activation_file);

        let mut controllers: HashMap<u64, Controller> = HashMap::new();
        let all_controller_settings: Vec<&ControllerSettings> =
            self.config.controllers.values().collect();

        let ratios: Vec<f32> = all_controller_settings
            .iter()
            .map(|controller_settings| controller_settings.ratio)
            .collect();

        info!("Controller ratios: {:?}", ratios);
        let dist = WeightedIndex::new(&ratios).unwrap();
        let mut rng = thread_rng();

        for (controller_id, activation_data) in controller_activations.iter() {
            let controller_timing = Self::convert_activation_to_timing(&activation_data);
            let controller_settings = match all_controller_settings.get(dist.sample(&mut rng)) {
                Some(controller_setting) => *controller_setting,
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
        info!("Number of Controllers: {}", controllers.len());
        return controllers;
    }

    pub(crate) fn convert_activation_to_timing(activation: &data_io::Activation) -> Timing {
        let mut activation_times: [Option<u64>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        let mut deactivation_times: [Option<u64>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        for (i, start_time) in activation.0.iter().enumerate() {
            activation_times[i] = Some(*start_time);
        }
        for (i, end_time) in activation.1.iter().enumerate() {
            deactivation_times[i] = Some(*end_time);
        }
        Timing {
            activation: activation_times,
            deactivation: deactivation_times,
        }
    }

    pub(crate) fn get_duration(&self) -> u64 {
        return self.config.simulation_settings.sim_duration;
    }
}
