use pavenet_models::node_pool::episode::EpisodeInfo;
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct DynamicConfig {
    pub episodes: Vec<EpisodeInfo>,
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
