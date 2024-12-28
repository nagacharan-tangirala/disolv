pub use std::path::PathBuf;

use disolv_core::bucket::TimeMS;
use disolv_output::logger::initiate_logger;
use disolv_output::ui::SimUIMetadata;

use crate::positions::reader::TraceReader;
use crate::positions::writer::TraceWriter;
use crate::simulation::config::Config;

pub(crate) struct TraceParser {
    pub(crate) step_size: TimeMS,
    pub(crate) duration: TimeMS,
    config_path: PathBuf,
    config: Config,
    reader: TraceReader,
    writer: TraceWriter,
}

impl TraceParser {
    pub(crate) fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            duration: config.timing_settings.duration,
            step_size: config.timing_settings.step_size,
            reader: TraceReader::new(&config.position_files),
            writer: TraceWriter::new(&config.output_settings),
            config,
            config_path,
        }
    }

    pub(crate) fn build_trace_metadata(&self) -> SimUIMetadata {
        SimUIMetadata {
            scenario: "trace_parser".to_string(),
            input_file: self.config.position_files.trace.to_string(),
            output_path: self.config.output_settings.output_path.to_string(),
            log_path: self.config.log_settings.log_path.clone(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        initiate_logger(&self.config_path, &self.config.log_settings, None);
        self.reader.initialize();
    }

    pub(crate) fn parse_positions_at(&self, time_ms: TimeMS) {}

    pub(crate) fn complete(self) {
        self.writer.flush();
    }
}
