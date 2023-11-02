use pavenet_core::bucket::TimeS;
use pavenet_core::entity::class::NodeClass;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::rules::RuleSettings;
use pavenet_node::models::compose::ComposerSettings;
use pavenet_node::models::latency::LatencyConfig;
use pavenet_node::models::linker::LinkerSettings;
use pavenet_node::models::respond::ResponderSettings;
use pavenet_node::models::select::SelectorSettings;
use pavenet_node::models::space::{FieldSettings, MobilitySettings};
use serde::Deserialize;
use std::path::PathBuf;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub field_settings: FieldSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub nodes: Vec<NodeSettings>,
    pub tx_rules: Vec<RuleSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub sim_name: String,
    pub sim_duration: TimeS,
    pub sim_step_size: TimeS,
    pub sim_streaming_step: TimeS,
    pub sim_seed: u64,
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
    pub output_step: TimeS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeSettings {
    pub node_type: NodeType,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<NodeClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeClassSettings {
    pub node_share: f32,
    pub node_class: NodeClass,
    pub latency: LatencyConfig,
    pub composer: Option<ComposerSettings>,
    pub selector: Option<SelectorSettings>,
    pub responder: Option<ResponderSettings>,
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
