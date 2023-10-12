use pavenet_core::enums::{DataType, MobilityType, NodeType, TransferMode};
use pavenet_core::types::TimeStamp;
use pavenet_models::node::composer::ComposerSettings;
use pavenet_models::node::responder::ResponderSettings;
use pavenet_models::node::simplifier::SimplifierSettings;
use pavenet_models::pool::linker::LinkerSettings;
use pavenet_models::pool::space::{FieldSettings, SpaceSettings};
use serde_derive::Deserialize;
use std::path::PathBuf;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub field_settings: FieldSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub episode_settings: Option<EpisodeSettings>,
    pub nodes: Vec<NodeSettings>,
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
pub struct NodeSettings {
    pub node_type: NodeType,
    pub power_file: String,
    pub space: SpaceSettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<NodeClassSettings>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct NodeClassSettings {
    pub node_share: f32,
    pub node_class: u32,
    pub node_order: i32,
    pub composer: Option<ComposerSettings>,
    pub simplifier: Option<SimplifierSettings>,
    pub responder: Option<ResponderSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EpisodeSettings {
    pub episode_file: String,
}

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

mod tests {
    #[cfg(test)]
    fn test_base_config_reader() {
        let base_config_file = "../test/data/test_config.toml";
        let config_reader = super::BaseConfigReader::new(&base_config_file);
        let base_config = config_reader.parse().unwrap();
        println!("{:?}", base_config);
    }
}
