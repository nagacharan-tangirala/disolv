use disolv_core::agent::AgentOrder;
use disolv_core::bucket::TimeMS;
use crate::linker::LinkerSettings;
use crate::space::{FieldSettings, MobilitySettings};
use disolv_models::device::compose::ComposerSettings;
use disolv_models::device::energy::EnergySettings;
use disolv_models::device::hardware::StorageSettings;
use disolv_models::device::reply::ReplierSettings;
use disolv_models::device::select::SelectorSettings;
use disolv_models::device::types::{DeviceClass, DeviceType};
use disolv_models::net::radio::ActionSettings;
use disolv_models::net::slice::SliceSettings;
use disolv_output::result::OutputSettings;
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
pub struct LogSettings {
    pub log_path: String,
    pub log_level: String,
    pub log_file_name: String,
    pub log_overwrite: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentSettings {
    pub agent_type: DeviceType,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<AgentClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub slice: Vec<SliceSettings>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct AgentClassSettings {
    pub agent_share: f32,
    pub agent_class: DeviceClass,
    pub agent_order: AgentOrder,
    pub composer: ComposerSettings,
    pub selector: Vec<SelectorSettings>,
    pub replier: ReplierSettings,
    pub energy: EnergySettings,
    pub storage: StorageSettings,
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
