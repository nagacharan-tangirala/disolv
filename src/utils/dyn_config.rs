use crate::reader::activation::{DeviceId, TimeStamp};
use crate::utils::config::{
    AggregatorSettings, BSLinkerSettings, ComposerSettings, ControllerLinkerSettings,
    DataSourceSettings, DeviceType, RSULinkerSettings, ResponderSettings, SimplifierSettings,
    VehicleLinkerSettings,
};
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub(crate) enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Location {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) enum EpisodeType {
    #[default]
    Persistent,
    Temporary,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DynamicConfig {
    pub(crate) episodes: Vec<EpisodeInfo>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct EpisodeInfo {
    pub(crate) time_stamp: TimeStamp,
    pub(crate) episode_type: EpisodeType,
    pub(crate) duration: Option<TimeStamp>,
    pub(crate) device_type: Option<DeviceType>,
    pub(crate) device_class: Option<u32>,
    pub(crate) device_list: Option<Vec<DeviceId>>,
    pub(crate) data_sources: Option<DataSourceSettings>,
    pub(crate) veh_linker: Option<VehicleLinkerSettings>,
    pub(crate) rsu_linker: Option<RSULinkerSettings>,
    pub(crate) bs_linker: Option<BSLinkerSettings>,
    pub(crate) controller_linker: Option<ControllerLinkerSettings>,
    pub(crate) composer: Option<ComposerSettings>,
    pub(crate) simplifier: Option<SimplifierSettings>,
    pub(crate) responder: Option<ResponderSettings>,
    pub(crate) aggregator: Option<AggregatorSettings>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ResetEpisodeInfo {
    time_stamp: TimeStamp,
    device_type: Option<DeviceType>,
    device_class: Option<u32>,
    device_list: Option<Vec<DeviceId>>,
}

pub(crate) struct DynamicConfigReader {
    file_path: PathBuf,
}

impl DynamicConfigReader {
    pub(crate) fn new(file_name: &str) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub(crate) fn parse(&self) -> Result<DynamicConfig, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: DynamicConfig = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
