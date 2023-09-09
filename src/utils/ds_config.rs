use serde_derive::Deserialize;
use std::collections::HashMap;
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
    Other,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AllDataSources {
    pub(crate) vehicle_sources: HashMap<String, HashMap<String, DataSourceSettings>>,
    pub(crate) rsu_sources: HashMap<String, HashMap<String, DataSourceSettings>>,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
pub(crate) struct DataSourceSettings {
    pub(crate) data_type: SensorType,
    pub(crate) data_counts: u16,
    pub(crate) unit_size: f32,
    pub(crate) target_type: DeviceType,
}

pub(crate) struct DSConfigReader {
    file_path: PathBuf,
}

impl DSConfigReader {
    pub(crate) fn new(file_name: &PathBuf) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub(crate) fn parse(&self) -> Result<AllDataSources, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: AllDataSources = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
