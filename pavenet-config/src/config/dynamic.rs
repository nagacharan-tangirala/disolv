use crate::config::base::{
    ComposerSettings, DeviceId, DeviceType, LinkerSettings, ResponderSettings, SimplifierSettings,
    SourceSettings, TimeStamp,
};
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone, Default)]
pub enum EpisodeType {
    #[default]
    Persistent,
    Temporary,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DynamicConfig {
    pub episodes: Vec<EpisodeInfo>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct EpisodeInfo {
    pub time_stamp: TimeStamp,
    pub episode_type: EpisodeType,
    pub duration: Option<TimeStamp>,
    pub device_type: Option<DeviceType>,
    pub device_class: Option<u32>,
    pub device_list: Option<Vec<DeviceId>>,
    pub data_sources: Option<SourceSettings>,
    pub linker_settings: Option<LinkerSettings>,
    pub composer: Option<ComposerSettings>,
    pub simplifier: Option<SimplifierSettings>,
    pub responder: Option<ResponderSettings>,
}

#[derive(Clone, Debug, Default)]
pub struct ResetEpisodeInfo {
    time_stamp: TimeStamp,
    device_type: Option<DeviceType>,
    device_class: Option<u32>,
    device_list: Option<Vec<DeviceId>>,
}

pub struct DynamicConfigReader {
    file_path: PathBuf,
}

impl DynamicConfigReader {
    pub fn new(file_name: &str) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub fn parse(&self) -> Result<DynamicConfig, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: DynamicConfig = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
