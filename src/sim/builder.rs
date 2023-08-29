use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::sim::network::Network;
use crate::utils::config;
use crate::utils::constants::{BASE_STATIONS, CONTROLLERS, STREAM_TIME, VEHICLES};
use crate::utils::data_io::{CsvDataReader, ParquetDataReader};
use crate::utils::df_handler::{ActivationHandler, VehicleTraceHandler};
use krabmaga::hashbrown::HashMap;
use polars_core::frame::DataFrame;
use std::path::Path;

pub struct PavenetBuilder {
    config: config::Config,
    config_path: String,
    data_readers: HashMap<String, ParquetDataReader>,
}

impl PavenetBuilder {
    pub fn new(config_file: &str) -> Self {
        if !Path::new(config_file).exists() {
            println!("Configuration file is not found.");
        }
        let config_path = Path::new(config_file)
            .parent()
            .unwrap_or_else(|| {
                println!("Invalid directory for the configuration file");
                std::process::exit(1);
            })
            .to_str()
            .unwrap_or_else(|| {
                println!("Failed to convert the configuration file path to string");
                std::process::exit(1);
            })
            .to_string();

        let config_reader = config::ConfigReader::new(config_file);
        match config_reader.parse() {
            Ok(config_data) => Self {
                config: config_data,
                config_path,
                data_readers: HashMap::new(),
            },
            Err(e) => {
                println!("Error while parsing the configuration file: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn build(&mut self) -> Network {
        let vehicles = self.build_vehicles();
        let roadside_units = self.build_roadside_units();
        let base_stations = self.build_base_stations();
        let controllers = self.build_controllers();

        return Network::new(
            self.config.clone(),
            vehicles,
            roadside_units,
            base_stations,
            controllers,
        );
    }

    fn build_vehicles(&mut self) -> HashMap<i32, Vehicle> {
        // Get the vehicle activation data.
        // Read the trace data until the first streaming interval.
        // Create the vehicles and add them to the hashmap.
        // Return the hashmap.
        let activation_file_path =
            Path::new(&self.config_path).join(&self.config.input_files.vehicle_activations);

        let vehicle_activations = Self::read_device_activation(
            &self,
            VEHICLES,
            activation_file_path.to_str().unwrap_or(""),
        );
        let vehicle_traces = Self::read_vehicle_traces(&self);

        let mut vehicles: HashMap<i32, Vehicle> = HashMap::new();
        return vehicles;
    }

    fn read_vehicle_traces(&self) -> HashMap<i64, DataFrame> {
        let trace_file: &str = &self.config.input_files.vehicle_traces;
        let trace_file_path = Path::new(&self.config_path).join(trace_file);

        let mut trace_reader: ParquetDataReader =
            ParquetDataReader::new(trace_file_path.to_str().unwrap_or(""));
        let data_start = 0;
        let data_end = STREAM_TIME as i64 - 1;

        let trace_df = match trace_reader.read_data(data_start, data_end) {
            Ok(trace_df) => trace_df,
            Err(e) => {
                println!("Error while reading the vehicle traces: {}", e);
                std::process::exit(1);
            }
        };

        let mut trace_handler: VehicleTraceHandler = VehicleTraceHandler::new(trace_df);
        let vehicle_trace_dfs = match trace_handler.prepare_trace_dfs() {
            Ok(df_map) => df_map,
            Err(e) => {
                println!("Error while reading the vehicle activations: {}", e);
                std::process::exit(1);
            }
        };
        return vehicle_trace_dfs;
    }

    fn build_roadside_units(&self) -> HashMap<i32, RoadsideUnit> {
        let rsu_activations =
            Self::read_device_activation(&self, "rsu_id", &self.config.input_files.rsu_activations);

        let mut roadside_units: HashMap<i32, RoadsideUnit> = HashMap::new();
        return roadside_units;
    }

    fn build_base_stations(&self) -> HashMap<i32, BaseStation> {
        let base_station_activations = Self::read_device_activation(
            &self,
            BASE_STATIONS,
            &self.config.input_files.base_station_activations,
        );

        let mut base_stations: HashMap<i32, BaseStation> = HashMap::new();
        return base_stations;
    }

    fn build_controllers(&self) -> HashMap<i32, Controller> {
        let controller_activations = Self::read_device_activation(
            &self,
            CONTROLLERS,
            &self.config.input_files.controller_activations,
        );

        let mut controllers: HashMap<i32, Controller> = HashMap::new();
        return controllers;
    }

    fn read_device_activation(
        &self,
        device_name: &str,
        activations_file: &str,
    ) -> HashMap<i64, (Vec<i64>, Vec<i64>)> {
        let mut activation_data_reader: CsvDataReader = CsvDataReader::new(activations_file);

        let activation_df = match activation_data_reader.read_data() {
            Ok(activation_df) => activation_df,
            Err(e) => {
                println!(
                    "Error while reading the activations for {}: {}",
                    device_name, e
                );
                std::process::exit(1);
            }
        };

        let mut activation_handler = ActivationHandler::new(activation_df);
        let activation_dfs = match activation_handler.prepare_device_activations() {
            Ok(activation_dfs) => activation_dfs,
            Err(e) => {
                println!(
                    "Error while reading the activations for {}: {}",
                    device_name, e
                );
                std::process::exit(1);
            }
        };
        return activation_dfs;
    }

    pub fn get_duration(&self) -> u64 {
        return self.config.simulation_settings.sim_duration;
    }
}
