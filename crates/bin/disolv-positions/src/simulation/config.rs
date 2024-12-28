use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::agent::AgentKind;
use disolv_core::bucket::TimeMS;
use disolv_output::logger::LogSettings;

#[derive(Deserialize, Debug, Clone)]
pub struct TimingSettings {
    pub duration: TimeMS,
    pub step_size: TimeMS,
    pub streaming_step: TimeMS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PositionFiles {
    pub network: AgentKind,
    pub trace: String,
    pub trace_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_path: String,
    pub output_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub timing_settings: TimingSettings,
    pub position_files: PositionFiles,
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
