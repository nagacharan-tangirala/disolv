use serde_derive::Deserialize;
use std::collections::HashMap as Map;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Copy, Clone)]
pub(crate) enum DataTargetType {
    Vehicle,
    RSU,
    BaseStation,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub(crate) enum DataType {
    Image,
    Video,
    Lidar3D,
    Lidar2D,
    Radar,
    Other,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AllDataSources {
    pub(crate) vehicle_sources: Map<String, DataSourceSettings>,
    pub(crate) rsu_sources: Map<String, DataSourceSettings>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub(crate) struct DataSourceSettings {
    pub(crate) id: u8,
    pub(crate) data_type: DataType,
    pub(crate) data_counts: u16,
    pub(crate) data_size: f32,
    pub(crate) target_type: DataTargetType,
}

pub(crate) struct DSConfigReader {
    file_path: PathBuf,
}

impl DSConfigReader {
    pub(crate) fn new(file_name: &PathBuf) -> Self {
        let file_path = PathBuf::from(file_name);
        Self {
            file_path: file_path,
        }
    }

    pub(crate) fn parse(&self) -> Result<AllDataSources, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: AllDataSources = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
