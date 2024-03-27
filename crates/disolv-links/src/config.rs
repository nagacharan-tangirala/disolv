use log::debug;
use std::path::PathBuf;
use toml;

use crate::linker::{DeviceCount, LinkType, Radius};
use crate::reader::TraceType;
use disolv_core::bucket::TimeMS;
use disolv_models::device::types::DeviceType;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_settings: LogSettings,
    pub settings: Settings,
    pub link_settings: Vec<LinkSettings>,
    pub position_files: Vec<PositionFiles>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogSettings {
    pub log_path: String,
    pub log_level: String,
    pub log_file_name: String,
    pub log_overwrite: bool,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub threads: u32,
    pub start: TimeMS,
    pub end: TimeMS,
    pub step_size: TimeMS,
    pub output_type: String,
    pub output_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PositionFiles {
    pub device: DeviceType,
    pub trace_type: TraceType,
    pub position_file: String,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct LinkSettings {
    pub source: DeviceType,
    pub target: DeviceType,
    pub link_count: Option<DeviceCount>,
    pub link_radius: Option<Radius>,
    pub link_model: String,
    pub link_type: LinkType,
    pub links_file: String,
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
    return config;
}
