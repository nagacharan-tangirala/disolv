use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::bucket::TimeMS;
use disolv_output::logger::LogSettings;

#[derive(Deserialize, Debug, Clone)]
pub struct TimingSettings {
    pub duration: TimeMS,
    pub step_size: TimeMS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TraceSettings {
    pub input_trace: String,
    pub input_network: String,
    pub trace_type: String,
    pub time_conversion: TimeMS,
    pub output_trace: String,
    pub starting_id: u64,
    pub activation_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RSUSettings {
    pub input_network: String,
    pub placement_type: String,
    pub output_file: String,
    pub starting_id: u64,
    pub activation_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParserSettings {
    pub vehicle_traces: bool,
    pub rsu_placement: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_settings: LogSettings,
    pub trace_settings: TraceSettings,
    pub rsu_settings: RSUSettings,
    pub timing_settings: TimingSettings,
    pub parser_settings: ParserSettings,
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
