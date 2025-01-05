use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::bucket::TimeMS;
use disolv_output::logger::LogSettings;

#[derive(Deserialize, Debug, Clone)]
pub struct TimingSettings {
    pub duration: TimeMS,
    pub step_size: TimeMS,
    pub streaming_step: TimeMS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TraceSettings {
    pub input_trace: String,
    pub input_network: String,
    pub trace_type: String,
    pub time_conversion: TimeMS,
    pub output_trace: String,
    pub starting_id: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActivationSettings {
    pub activation_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_settings: LogSettings,
    pub trace_settings: TraceSettings,
    pub activation_settings: ActivationSettings,
    pub timing_settings: TimingSettings,
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
