use crate::config::base::{BaseConfig, BaseConfigReader};
use crate::config::logger;
use crate::scenario::device::Device;
use hashbrown::HashMap;
use log::{debug, info, warn};
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_engine::engine::core::Core;
use pavenet_engine::engine::nodes::Nodes;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PavenetBuilder {
    base_config: BaseConfig,
    config_path: PathBuf,
}

impl PavenetBuilder {
    pub fn new(base_config_file: &str) -> Self {
        if !Path::new(base_config_file).exists() {
            panic!("Configuration file is not found.");
        }
        let config_path = Path::new(base_config_file)
            .parent()
            .unwrap_or_else(|| {
                panic!("Invalid directory for the configuration file");
            })
            .to_path_buf();

        let config_reader = BaseConfigReader::new(&base_config_file);
        match config_reader.parse() {
            Ok(base_config) => Self {
                base_config,
                config_path,
            },
            Err(e) => {
                panic!("Error while parsing the base configuration file: {}", e);
            }
        }
    }

    pub fn build(&mut self) -> Core {
        self.initiate_logger();

        let start_time = TimeStamp::default();
        let end_time = TimeStamp::from(self.base_config.simulation_settings.sim_duration);
        let streaming_time =
            TimeStamp::from(self.base_config.simulation_settings.sim_streaming_step);

        let nodes = self.build_nodes();
        let node_collections = self.build_node_collections();

        Core::builder()
            .step(start_time)
            .end_step(end_time)
            .streaming_step(streaming_time)
            .nodes(nodes)
            .node_collections(node_collections)
            .build()
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

    pub fn build_nodes(&mut self) -> Nodes {
        let devices = self.build_devices();
        let mut node_impls = HashMap::with_capacity(devices.len());
        for device in devices.values() {
            node_impls.insert(device.node_info.id, device.node_impl.clone());
        }
        return Nodes::new(node_impls);
    }

    fn build_devices(&mut self) -> HashMap<NodeId, Device> {
        info!("Building devices...");
        let device_settings: &Vec<&DeviceSettings> = &self
            .base_config
            .devices
            .iter()
            .filter(|vs| vs.device_type == DeviceType::Vehicle)
            .collect();

        let models: DeviceModel = ModelBuilder::new()
            .with_composer(&vehicle_settings.composer)
            .with_simplifier(&vehicle_settings.simplifier)
            .with_power_schedule(power_schedule)
            .build();

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
