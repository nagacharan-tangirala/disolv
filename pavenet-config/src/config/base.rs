use super::enums::{DataType, MobilityType, NodeType, TransferMode};
use crate::types::ts::TimeStamp;
use serde_derive::Deserialize;
use std::path::PathBuf;

pub type PowerTimes = (Vec<TimeStamp>, Vec<TimeStamp>); // (on times, off times)

pub struct BaseConfigReader {
    file_path: PathBuf,
}

impl BaseConfigReader {
    pub fn new(file_name: &str) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub fn parse(&self) -> Result<BaseConfig, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: BaseConfig = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub field_settings: FieldSettings,
    pub node_settings: Vec<NodeSettings>,
    pub network_settings: NetworkSettings,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct NodeSettings {
    pub ratio: f32,
    pub node_type: NodeType,
    pub node_class: u32,
    pub node_order: i32,
    pub activation_file: String,
    pub mobility_settings: MapStateSettings,
    pub linker: LinkerSettings,
    pub composer: Option<ComposerSettings>,
    pub simplifier: Option<SimplifierSettings>,
    pub responder: Option<ResponderSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub sim_name: String,
    pub sim_duration: u64,
    pub sim_step: u64,
    pub sim_streaming_step: u64,
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
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<SourceSettings>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct SourceSettings {
    pub data_type: DataType,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct SimplifierSettings {
    pub name: String,
    pub compression_factor: Option<f32>,
    pub sampling_factor: Option<f32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MapStateSettings {
    pub mobility_type: MobilityType,
    pub is_streaming: bool,
    pub geo_data_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FieldSettings {
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub name: String,
    pub links: Vec<LinkConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkConfig {
    pub transfer_mode: TransferMode,
    pub target_device: NodeType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}
