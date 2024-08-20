use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct FlConfig {
    log_settings: LogSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogSettings {
    pub log_path: String,
    pub log_level: String,
    pub log_file_name: String,
    pub log_overwrite: bool,
}
