use serde_derive::Deserialize;
use std::collections::HashMap as Map;

pub(crate) struct ConfigReader {
    file_name: String,
}

impl ConfigReader {
    pub(crate) fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
        }
    }

    pub(crate) fn parse(&self) -> Result<Config, Box<dyn std::error::Error>> {
        println!("Parsing file: {}", self.file_name);
        let parsing_result = std::fs::read_to_string(&self.file_name)?;
        let config: Config = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) input_files: InputFiles,
    pub(crate) simulation_settings: SimSettings,
    pub(crate) log_settings: LogSettings,
    pub(crate) output_settings: OutputSettings,
    pub(crate) vehicles: Map<String, VehicleSettings>,
    pub(crate) roadside_units: Map<String, RSUSettings>,
    pub(crate) base_stations: Map<String, BaseStationSettings>,
    pub(crate) controllers: Map<String, ControllerSettings>,
    pub(crate) edge_orchestrator: EdgeOrchestratorSettings,
    pub(crate) cloud_orchestrator: CloudOrchestratorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct InputFiles {
    pub(crate) vehicle_traces: String,
    pub(crate) vehicle_activations: String,
    pub(crate) v2v_links: String,
    pub(crate) base_stations: String,
    pub(crate) base_station_activations: String,
    pub(crate) v2b_links: String,
    pub(crate) controllers: String,
    pub(crate) controller_activations: String,
    pub(crate) b2c_links: String,
    pub(crate) roadside_units: String,
    pub(crate) rsu_activations: String,
    pub(crate) v2r_links: String,
    pub(crate) r2b_links: String,
    pub(crate) r2r_links: String,
    pub(crate) data_source_config: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SimSettings {
    pub(crate) sim_name: String,
    pub(crate) sim_duration: u64,
    pub(crate) sim_step: u64,
    pub(crate) sim_streaming_step: u64,
    pub(crate) dimension_x_max: f32,
    pub(crate) dimension_y_max: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LogSettings {
    pub(crate) log_path: String,
    pub(crate) log_level: String,
    pub(crate) log_file_name: String,
    pub(crate) log_overwrite: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct OutputSettings {
    pub(crate) output_path: String,
    pub(crate) output_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MobilitySettings {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ComposerSettings {
    pub(crate) name: String,
    pub(crate) data_source_list: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct CollectorSettings {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AggregatorSettings {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SimplifierSettings {
    pub(crate) name: String,
    pub(crate) compression_factor: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct VehicleSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) mobility: MobilitySettings,
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
    pub(crate) collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RSUSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) mobility: MobilitySettings,
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
    pub(crate) collector: CollectorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BaseStationSettings {
    pub(crate) storage: f32,
    pub(crate) mobility: MobilitySettings,
    pub(crate) collector: CollectorSettings,
    pub(crate) aggregator: AggregatorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ControllerSettings {
    pub(crate) storage: f32,
    pub(crate) mobility: MobilitySettings,
    pub(crate) collector: CollectorSettings,
    pub(crate) aggregator: AggregatorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AllocatorStrategy {
    pub(crate) strategy: Map<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EdgeOrchestratorSettings {
    pub(crate) v2v_allocator: AllocatorStrategy,
    pub(crate) v2b_allocator: AllocatorStrategy,
    pub(crate) v2r_allocator: AllocatorStrategy,
    pub(crate) r2b_allocator: AllocatorStrategy,
    pub(crate) r2r_allocator: AllocatorStrategy,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct CloudOrchestratorSettings {
    pub(crate) b2c_allocator: AllocatorStrategy,
}
