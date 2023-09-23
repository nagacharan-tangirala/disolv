use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::runtime::ConfigErrors;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::PathBuf;

pub fn setup_logging(log_level: &str, log_file_path: PathBuf) -> Result<Config, ConfigErrors> {
    let log_level = get_logging_level(log_level);

    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y.%m.%d %H:%M:%S)} | {({l}):5.5} | {({f}:{L}):>30.30} — {m}{n}",
        )))
        .build(log_file_path)
        .unwrap();

    return Config::builder()
        .appender(Appender::builder().build("x", Box::new(log_file)))
        .build(Root::builder().appender("x").build(log_level));
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
