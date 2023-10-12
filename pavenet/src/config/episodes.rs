use crate::scenario::episode::EpisodeInfo;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct EpisodeList {
    pub episodes: Vec<EpisodeInfo>,
}

pub struct EpisodeReader {
    file_path: PathBuf,
}

impl EpisodeReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub fn parse(&self) -> Result<EpisodeList, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: EpisodeList = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
