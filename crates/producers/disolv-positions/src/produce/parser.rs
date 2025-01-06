pub use std::path::PathBuf;

use disolv_core::bucket::TimeMS;
use disolv_output::logger::initiate_logger;
use disolv_output::result::ResultWriter;
use disolv_output::ui::SimUIMetadata;

use crate::activation::writer::ActivationWriter;
use crate::produce::config::Config;
use crate::vehicles::trace::TraceReader;
use crate::vehicles::writer::TraceWriter;

pub(crate) struct TraceParser {
    pub(crate) step_size: TimeMS,
    pub(crate) duration: TimeMS,
    config_path: PathBuf,
    config: Config,
    trace_reader: TraceReader,
    trace_writer: TraceWriter,
    activation_writer: ActivationWriter,
}

impl TraceParser {
    pub(crate) fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            duration: config.timing_settings.duration,
            step_size: config.timing_settings.step_size,
            trace_reader: TraceReader::new(&config.trace_settings),
            trace_writer: TraceWriter::new(&config.trace_settings),
            activation_writer: ActivationWriter::new(&config.activation_settings),
            config,
            config_path,
        }
    }

    pub(crate) fn build_trace_metadata(&self) -> SimUIMetadata {
        SimUIMetadata {
            scenario: "trace_parser".to_string(),
            input_file: self.config.trace_settings.input_trace.to_string(),
            output_path: self.config.trace_settings.output_trace.to_string(),
            log_path: self.config.log_settings.log_path.clone(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        initiate_logger(&self.config_path, &self.config.log_settings, None);
        self.trace_reader.initialize();
    }

    pub(crate) fn parse_positions_at(&mut self, time_ms: TimeMS) {
        if let Some(trace_data) = self.trace_reader.read_data(time_ms) {
            self.activation_writer
                .determine_activations(&trace_data, time_ms);
            trace_data
                .into_iter()
                .for_each(|trace_info| self.trace_writer.store_info(trace_info));
        }
        self.trace_writer.write_to_file();
        self.activation_writer.write_to_file();
    }

    pub(crate) fn complete(self) {
        self.trace_writer.flush();
        self.activation_writer.flush();
    }
}
