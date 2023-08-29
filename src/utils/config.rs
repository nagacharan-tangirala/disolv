use serde_derive::Deserialize;
use std::collections::HashMap as Map;

pub struct ConfigReader {
    file_name: String,
}

impl ConfigReader {
    pub fn new(file_name: String) -> Self {
        Self { file_name }
    }

    pub fn parse(&self) -> Result<Config, Box<dyn std::error::Error>> {
        println!("Parsing file: {}", self.file_name);
        let parsing_result = std::fs::read_to_string(&self.file_name)?;
        let config: Config = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub input_files: InputFiles,
    pub simulation_settings: SimSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub vehicles: Map<String, VehicleSettings>,
    pub roadside_units: Map<String, RSUSettings>,
    pub base_stations: Map<String, BaseStationSettings>,
    pub controllers: Map<String, ControllerSettings>,
    pub edge_orchestrator: EdgeOrchestratorSettings,
    pub cloud_orchestrator: CloudOrchestratorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InputFiles {
    pub vehicle_traces: String,
    pub vehicle_activations: String,
    pub v2v_links: String,
    pub base_stations: String,
    pub base_station_activations: String,
    pub v2b_links: String,
    pub controllers: String,
    pub controller_activations: String,
    pub b2c_links: String,
    pub roadside_units: String,
    pub rsu_activations: String,
    pub v2r_links: String,
    pub r2b_links: String,
    pub r2r_links: String,
    pub data_source_config: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub sim_name: String,
    pub sim_duration: u64,
    pub sim_step: u64,
    pub sim_streaming_step: u64,
    pub dimension_x_max: f32,
    pub dimension_y_max: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogSettings {
    pub log_path: String,
    pub log_level: String,
    pub log_file_name: String,
    pub log_overwrite: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_path: String,
    pub output_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MobilitySettings {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComposerSettings {
    pub name: String,
    pub data_source_list: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CollectorSettings {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimplifierSettings {
    pub name: String,
    pub compression_factor: f32,
    pub retention_factor: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VehicleSettings {
    pub ratio: f32,
    pub storage: f32,
    pub mobility: MobilitySettings,
    pub composer: ComposerSettings,
    pub simplifier: SimplifierSettings,
    pub collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RSUSettings {
    pub ratio: f32,
    pub storage: f32,
    pub mobility: MobilitySettings,
    pub composer: ComposerSettings,
    pub simplifier: SimplifierSettings,
    pub collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BaseStationSettings {
    pub storage: f32,
    pub mobility: MobilitySettings,
    pub composer: ComposerSettings,
    pub collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ControllerSettings {
    pub storage: f32,
    pub mobility: MobilitySettings,
    pub composer: ComposerSettings,
    pub collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AllocatorStrategy {
    pub name: String,
    pub parameters: Map<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EdgeOrchestratorSettings {
    pub name: String,
    pub v2v_allocator: AllocatorStrategy,
    pub v2b_allocator: AllocatorStrategy,
    pub v2r_allocator: AllocatorStrategy,
    pub r2b_allocator: AllocatorStrategy,
    pub r2r_allocator: AllocatorStrategy,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CloudOrchestratorSettings {
    pub name: String,
    pub b2c_allocator: AllocatorStrategy,
}
