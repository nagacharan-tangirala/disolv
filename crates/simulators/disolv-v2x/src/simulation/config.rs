use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::agent::{AgentClass, AgentKind, AgentOrder};
use disolv_core::bucket::TimeMS;
use disolv_models::device::directions::CommDirections;
use disolv_models::net::radio::ActionSettings;
use disolv_output::logger::LogSettings;
use disolv_output::result::OutputSettings;

use crate::models::compose::ComposerSettings;
use crate::models::message::DataType;
use crate::models::network::SliceSettings;
use crate::models::select::SelectorSettings;
use crate::v2x::linker::LinkerSettings;
use crate::v2x::space::{FieldSettings, MobilitySettings};

#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub field_settings: FieldSettings,
    pub network_settings: NetworkSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub agents: Vec<AgentSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub scenario: String,
    pub duration: TimeMS,
    pub step_size: TimeMS,
    pub streaming_interval: TimeMS,
    pub seed: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentSettings {
    pub agent_type: AgentKind,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<AgentClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub slice: Vec<SliceSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentClassSettings {
    pub agent_share: f32,
    pub agent_class: AgentClass,
    pub agent_order: AgentOrder,
    pub composer: ComposerSettings,
    pub selector: Vec<SelectorSettings>,
    pub actions: Option<Vec<ActionSettings<DataType>>>,
    pub directions: Vec<CommDirections>,
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
