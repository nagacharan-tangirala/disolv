use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::agent::AgentKind;
use disolv_core::bucket::TimeMS;
use disolv_output::logger::LogSettings;

use crate::links::linker::{DeviceCount, LinkType, Radius};
use crate::links::reader::TraceType;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub start: TimeMS,
    pub end: TimeMS,
    pub step_size: TimeMS,
    pub output_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PositionFiles {
    pub device: AgentKind,
    pub trace_type: TraceType,
    pub position_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkSettings {
    pub source: AgentKind,
    pub target: AgentKind,
    pub link_count: Option<DeviceCount>,
    pub link_radius: Option<Radius>,
    pub link_model: String,
    pub link_type: LinkType,
    pub links_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_settings: LogSettings,
    pub settings: Settings,
    pub link_settings: Vec<LinkSettings>,
    pub position_files: Vec<PositionFiles>,
}

pub(crate) fn read_config(file_path: &PathBuf) -> Config {
    let input_toml = match std::fs::read_to_string(file_path) {
        Ok(parsed_string) => parsed_string,
        Err(_) => panic!("Failed to read input TOML file"),
    };
    let config: Config = match toml::from_str(&input_toml) {
        Ok(config) => config,
        Err(_) => panic!("Invalid toml file given"),
    };
    config
}
