use crate::config::LogSettings;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::runtime::ConfigErrors;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::fs;
use std::path::{Path, PathBuf};

pub fn setup_logging(log_level: &str, log_file_path: PathBuf) -> Result<Config, ConfigErrors> {
    let log_level = get_logging_level(log_level);
    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y.%m.%d %H:%M:%S)} | {({l}):5.5} | {({f}:{L}):>40.40} — {m}{n}",
        )))
        .build(log_file_path)
        .unwrap();

    Config::builder()
        .appender(Appender::builder().build("x", Box::new(log_file)))
        .build(Root::builder().appender("x").build(log_level))
}

fn get_logging_level(log_level: &str) -> LevelFilter {
    match log_level {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    }
}

pub fn initiate_logger(config_path: &Path, log_settings: &LogSettings) {
    let log_settings = log_settings.clone();
    let log_level = log_settings.log_level;
    let log_path = config_path.join(log_settings.log_path);

    if !log_path.exists() {
        fs::create_dir_all(&log_path)
            .unwrap_or_else(|_| panic!("Error while creating the log directory"));
    }

    let log_file_path = log_path.join(log_settings.log_file_name);
    if log_file_path.exists() {
        fs::remove_file(&log_file_path)
            .unwrap_or_else(|_| panic!("Error while clearing the log file"));
    }

    let logger_config = match setup_logging(&log_level, log_file_path) {
        Ok(logger_config) => logger_config,
        Err(e) => {
            panic!("Error while configuring the logger: {}", e);
        }
    };

    match log4rs::init_config(logger_config) {
        Ok(_) => {}
        Err(e) => {
            panic!("Error while initializing logger with config: {}", e);
        }
    };
}
