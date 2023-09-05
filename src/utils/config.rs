use serde_derive::Deserialize;
use std::collections::HashMap; // krabmaga::hashbrown::HashMap cannot be deserialized.
use std::path::PathBuf;

pub(crate) struct ConfigReader {
    file_path: PathBuf,
}

impl ConfigReader {
    pub(crate) fn new(file_name: &str) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub(crate) fn parse(&self) -> Result<Config, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: Config = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) position_files: PositionFiles,
    pub(crate) activation_files: ActivationFiles,
    pub(crate) link_files: LinkFiles,
    pub(crate) data_source_config_file: DataSourceConfigFile,
    pub(crate) simulation_settings: SimSettings,
    pub(crate) log_settings: LogSettings,
    pub(crate) output_settings: OutputSettings,
    pub(crate) vehicles: HashMap<String, VehicleSettings>,
    pub(crate) roadside_units: HashMap<String, RSUSettings>,
    pub(crate) base_stations: HashMap<String, BaseStationSettings>,
    pub(crate) controllers: HashMap<String, ControllerSettings>,
    pub(crate) mesh_links: MeshLinkSettings,
    pub(crate) infra_links: InfraLinkSettings,
    pub(crate) field_settings: FieldSettings,
    pub(crate) network_settings: NetworkSettings,
    pub(crate) trace_flags: TraceFlags,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PositionFiles {
    pub(crate) vehicle_positions: String,
    pub(crate) rsu_positions: String,
    pub(crate) bs_positions: String,
    pub(crate) controller_positions: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ActivationFiles {
    pub(crate) vehicle_activations: String,
    pub(crate) rsu_activations: String,
    pub(crate) base_station_activations: String,
    pub(crate) controller_activations: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LinkFiles {
    pub(crate) v2v_links: String,
    pub(crate) v2bs_links: String,
    pub(crate) v2rsu_links: String,
    pub(crate) rsu2rsu_links: String,
    pub(crate) rsu2bs_links: String,
    pub(crate) bs2c_links: String,
    pub(crate) c2c_links: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DataSourceConfigFile {
    pub(crate) config_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SimSettings {
    pub(crate) sim_name: String,
    pub(crate) sim_duration: u64,
    pub(crate) sim_step: u64,
    pub(crate) sim_streaming_step: u64,
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
pub(crate) struct ComposerSettings {
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
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RSUSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BaseStationSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) aggregator: AggregatorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ControllerSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) aggregator: AggregatorSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AllocatorStrategy {
    pub(crate) strategy: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MeshLinkSettings {
    pub(crate) v2v_allocator: AllocatorStrategy,
    pub(crate) v2r_allocator: AllocatorStrategy,
    pub(crate) r2r_allocator: AllocatorStrategy,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct InfraLinkSettings {
    pub(crate) v2b_allocator: AllocatorStrategy,
    pub(crate) r2b_allocator: AllocatorStrategy,
    pub(crate) b2c_allocator: AllocatorStrategy,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct FieldSettings {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct TraceFlags {
    pub(crate) vehicle: bool,
    pub(crate) roadside_unit: bool,
    pub(crate) base_station: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct NetworkSettings {}
