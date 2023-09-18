use crate::reader::activation::{DeviceId, TimeStamp};
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

#[derive(Deserialize, Debug, Clone)]
pub(crate) enum ActionType {
    ModifyData,
    ModifyDevice,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DynamicConfig {
    pub(crate) parameter_set: Vec<ParameterSet>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ParameterSet {
    pub(crate) time_stamp: TimeStamp,
    pub(crate) action_type: ActionType,
    pub(crate) data_types: Option<Vec<DataType>>,
    pub(crate) device_id: Option<Vec<DeviceId>>,
    pub(crate) location: Option<Vec<Location>>,
    pub(crate) duration: Option<TimeStamp>,
    pub(crate) frequency: Option<TimeStamp>,
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
