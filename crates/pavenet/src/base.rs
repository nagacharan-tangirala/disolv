use pavenet_core::entity::{NodeClass, NodeOrder, NodeType};
use pavenet_core::radio::ActionSettings;
use pavenet_engine::bucket::TimeMS;
use pavenet_models::compose::ComposerSettings;
use pavenet_models::reply::ReplierSettings;
use pavenet_models::select::SelectorSettings;
use pavenet_models::slice::SliceSettings;
use pavenet_node::linker::LinkerSettings;
use pavenet_node::space::{FieldSettings, MobilitySettings};
use pavenet_output::result::OutputSettings;
use serde::Deserialize;
use std::path::PathBuf;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub field_settings: FieldSettings,
    pub network_settings: NetworkSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub nodes: Vec<NodeSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub sim_name: String,
    pub sim_duration: TimeMS,
    pub sim_step_size: TimeMS,
    pub sim_streaming_step: TimeMS,
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
pub struct NodeSettings {
    pub node_type: NodeType,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<NodeClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub slice: Vec<SliceSettings>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct NodeClassSettings {
    pub node_share: f32,
    pub node_class: NodeClass,
    pub node_order: NodeOrder,
    pub composer: ComposerSettings,
    pub selector: Vec<SelectorSettings>,
    pub replier: ReplierSettings,
    pub actions: Option<Vec<ActionSettings>>,
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

#[cfg(test)]
mod tests {
    #[cfg(test)]
    fn test_base_config_reader() {
        let base_config_file = "../test/data/test_config.toml";
        let config_reader = super::BaseConfigReader::new(&base_config_file);
        let base_config = config_reader.parse().unwrap();
        println!("{:?}", base_config);
    }
}
