use crate::config::base::{DeviceSettings, DeviceType};
use crate::config::constants::ARRAY_SIZE;
use crate::config::{base, dynamic, logger};
use pavenet_core::devices::bs::BaseStation;
use crate::device::common::Timing;
use pavenet_core::devices::controller::Controller;
use pavenet_core::devices::rsu::RoadsideUnit;
use pavenet_core::devices::vehicle::Vehicle;
use crate::reader::activation;
use crate::reader::activation::{Activation, DeviceId, TimeStamp};
use crate::sim::core::Core;
use crate::sim::field::DeviceField;
use crate::sim::vanet::Vanet;
use krabmaga::hashbrown::HashMap;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

pub struct PavenetBuilder {
    base_config: base::BaseConfig,
    dyn_config: dynamic::DynamicConfig,
    config_path: PathBuf,
}

impl PavenetBuilder {
    pub fn new(base_config_file: &str, dyn_config_file: &str) -> Self {
        if !Path::new(base_config_file).exists() {
            panic!("Configuration file is not found.");
        }
        let config_path = Path::new(base_config_file)
            .parent()
            .unwrap_or_else(|| {
                panic!("Invalid directory for the configuration file");
            })
            .to_path_buf();

        let dyn_config_reader = dynamic::DynamicConfigReader::new(&dyn_config_file);
        let dyn_config = match dyn_config_reader.parse() {
            Ok(dyn_config) => dyn_config,
            Err(e) => {
                panic!("Error while parsing the dynamic configuration file: {}", e);
            }
        };

        let config_reader = base::BaseConfigReader::new(&base_config_file);
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

    pub fn build(&mut self) -> Core {
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

    pub fn initiate_logger(&self) {
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
            &self.config_path,
            self.base_config.simulation_settings.sim_streaming_step,
        );
    }

    fn build_empty_vanet(&self) -> Vanet {
        return Vanet::new(
            &self.config_path,
            &self.base_config.network_settings,
            self.base_config.simulation_settings.sim_streaming_step,
        );
    }

    fn build_vehicles(&mut self) -> HashMap<DeviceId, Vehicle> {
        info!("Building vehicles...");
        let device_settings: &Vec<&DeviceSettings> = &self
            .base_config
            .devices
            .iter()
            .filter(|vs| vs.device_type == DeviceType::Vehicle)
            .collect();

        if device_settings.len() == 0 {
            warn!("No vehicle is defined in the configuration file.");
            return HashMap::new();
        }

        let mut vehicles: HashMap<DeviceId, Vehicle> = HashMap::new();
        for device_setting in device_settings.into_iter() {
            let activation_file =
                Path::new(&self.config_path).join(&device_setting.activation_file);
            if activation_file.exists() == false {
                panic!("Vehicle activation file is not found.");
            }
            let activations: HashMap<DeviceId, Activation> =
                activation::read_activation_data(activation_file);

            vehicles.reserve(activations.len());
            for (vehicle_id, activation_data) in activations.iter() {
                let vehicle_timing = Self::convert_activation_to_timing(&activation_data);
                let new_vehicle = Vehicle::new(*vehicle_id, vehicle_timing, &device_setting);
                vehicles.entry(*vehicle_id).or_insert(new_vehicle);
            }
        }

        info!("Done! Number of vehicles: {}", vehicles.len());
        return vehicles;
    }

    fn build_roadside_units(&self) -> HashMap<DeviceId, RoadsideUnit> {
        info!("Building roadside units...");
        let device_settings: &Vec<&DeviceSettings> = &self
            .base_config
            .devices
            .iter()
            .filter(|vs| vs.device_type == DeviceType::RSU)
            .collect();

        if device_settings.len() == 0 {
            warn!("No RSU is defined in the configuration file.");
            return HashMap::new();
        }

        let mut roadside_units: HashMap<DeviceId, RoadsideUnit> = HashMap::new();
        for device_setting in device_settings.into_iter() {
            let activation_file =
                Path::new(&self.config_path).join(&device_setting.activation_file);
            if activation_file.exists() == false {
                panic!("RSU activation file is not found.");
            }
            let rsu_activations: HashMap<DeviceId, Activation> =
                activation::read_activation_data(activation_file);

            roadside_units.reserve(rsu_activations.len());
            for (rsu_id, activation_data) in rsu_activations.iter() {
                let rsu_timing = Self::convert_activation_to_timing(&activation_data);
                let new_rsu = RoadsideUnit::new(*rsu_id, rsu_timing, device_setting);
                roadside_units.entry(*rsu_id).or_insert(new_rsu);
            }
        }

        info!("Done! Number of Roadside Units: {}", roadside_units.len());
        return roadside_units;
    }

    fn build_base_stations(&self) -> HashMap<u64, BaseStation> {
        info!("Building base stations...");
        let device_settings: &Vec<&DeviceSettings> = &self
            .base_config
            .devices
            .iter()
            .filter(|vs| vs.device_type == DeviceType::BaseStation)
            .collect();

        if device_settings.len() == 0 {
            warn!("No base station is defined in the configuration file.");
            return HashMap::new();
        }

        let mut base_stations: HashMap<DeviceId, BaseStation> = HashMap::new();
        for device_setting in device_settings.iter() {
            let activation_file =
                Path::new(&self.config_path).join(&device_setting.activation_file);
            if activation_file.exists() == false {
                panic!("Base station activation file is not found.");
            }
            let bs_activations: HashMap<DeviceId, Activation> =
                activation::read_activation_data(activation_file);

            base_stations.reserve(bs_activations.len());
            for (bs_id, activation_data) in bs_activations.iter() {
                let bs_timing = Self::convert_activation_to_timing(&activation_data);
                let new_base_station = BaseStation::new(*bs_id, bs_timing, device_setting);
                if let Some(value) = base_stations.insert(*bs_id, new_base_station) {
                    panic!("Duplicate base station id: {}", value.id);
                }
            }
        }
        info!("Done! Number of Base stations: {}", base_stations.len());
        return base_stations;
    }

    fn build_controllers(&self) -> HashMap<u64, Controller> {
        info!("Building controllers...");
        let device_settings: &Vec<&DeviceSettings> = &self
            .base_config
            .devices
            .iter()
            .filter(|vs| vs.device_type == DeviceType::Controller)
            .collect();

        if device_settings.len() == 0 {
            warn!("No controller is defined in the configuration file.");
            return HashMap::new();
        }

        let mut controllers: HashMap<DeviceId, Controller> = HashMap::new();
        for device_setting in device_settings.into_iter() {
            let activation_file =
                Path::new(&self.config_path).join(&device_setting.activation_file);
            if activation_file.exists() == false {
                panic!("Controller activation file is not found.");
            }
            let controller_activations: HashMap<DeviceId, Activation> =
                activation::read_activation_data(activation_file);

            controllers.reserve(controller_activations.len());
            for (controller_id, activation_data) in controller_activations.iter() {
                let controller_timing = Self::convert_activation_to_timing(&activation_data);
                let base_controller =
                    Controller::new(*controller_id, controller_timing, device_setting);
                if let Some(value) = controllers.insert(*controller_id, base_controller) {
                    panic!("Duplicate controller id: {}", value.id);
                }
            }
        }
        info!("Done! Number of Controllers: {}", controllers.len());
        return controllers;
    }

    pub fn convert_activation_to_timing(activation: &Activation) -> Timing {
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

    pub fn get_duration(&self) -> u64 {
        return self.base_config.simulation_settings.sim_duration;
    }
}
