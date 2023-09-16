use serde_derive::Deserialize;
use std::collections::HashMap; // krabmaga::hashbrown::HashMap cannot be deserialized.
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum DeviceType {
    #[default]
    None = 0,
    Vehicle,
    RSU,
    BaseStation,
    Controller,
}

#[derive(Deserialize, Default, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub(crate) enum SensorType {
    #[default]
    None = 0,
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

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
    pub(crate) geo_data_files: GeoDataFiles,
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
    pub(crate) field_settings: FieldSettings,
    pub(crate) network_settings: NetworkSettings,
    pub(crate) trace_flags: TraceFlags,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GeoDataFiles {
    pub(crate) vehicle_geo_data: String,
    pub(crate) rsu_geo_data: String,
    pub(crate) bs_geo_data: String,
    pub(crate) controller_geo_data: String,
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
    pub(crate) sampling_factor: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct VehicleSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
    pub(crate) linker: VehicleLinker,
    pub(crate) data_sources: DataSourceSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RSUSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) composer: ComposerSettings,
    pub(crate) simplifier: SimplifierSettings,
    pub(crate) linker: RSULinker,
    pub(crate) data_sources: DataSourceSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BaseStationSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) aggregator: AggregatorSettings,
    pub(crate) responder: ResponderSettings,
    pub(crate) linker: BSLinker,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ControllerSettings {
    pub(crate) ratio: f32,
    pub(crate) storage: f32,
    pub(crate) aggregator: AggregatorSettings,
    pub(crate) responder: ResponderSettings,
    pub(crate) linker: ControllerLinker,
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

#[derive(Deserialize, Default, Debug, Clone)]
pub(crate) struct DataSourceSettings {
    pub(crate) data_types: Vec<SensorType>,
    pub(crate) data_counts: Vec<u32>,
    pub(crate) unit_sizes: Vec<f32>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct NetworkSettings {}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ResponderSettings {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct VehicleLinker {
    pub(crate) name: String,
    pub(crate) mesh_range: f32,
    pub(crate) bs_range: f32,
    pub(crate) rsu_range: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct RSULinker {
    pub(crate) name: String,
    pub(crate) mesh_range: f32,
    pub(crate) bs_range: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct BSLinker {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct ControllerLinker {
    pub(crate) name: String,
}
