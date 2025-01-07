pub use std::path::PathBuf;

use disolv_core::bucket::TimeMS;
use disolv_output::logger::initiate_logger;
use disolv_output::ui::SimUIMetadata;

use crate::produce::config::Config;
use crate::rsu::junctions::RSUPlacement;
use crate::vehicles::trace::TraceHelper;

pub(crate) struct TraceParser {
    pub(crate) step_size: TimeMS,
    pub(crate) duration: TimeMS,
    config_path: PathBuf,
    config: Config,
    trace_helper: Option<TraceHelper>,
    rsu_placement: Option<RSUPlacement>,
    vehicles_flag: bool,
    rsu_flag: bool,
}

impl TraceParser {
    pub(crate) fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            duration: config.timing_settings.duration,
            step_size: config.timing_settings.step_size,
            trace_helper: None,
            rsu_placement: None,
            config_path,
            vehicles_flag: config.parser_settings.vehicle_traces,
            rsu_flag: config.parser_settings.rsu_placement,
            config,
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
        if self.vehicles_flag {
            let mut trace_helper = TraceHelper::new(&self.config.trace_settings);
            trace_helper.initialize();
            self.trace_helper = Some(trace_helper);
        }

        if self.rsu_flag {
            let mut rsu_placement =
                RSUPlacement::new(&self.config.rsu_settings, &self.config.timing_settings);
            rsu_placement.initialize();
            self.rsu_placement = Some(rsu_placement);
        }
    }

    pub(crate) fn parse_positions_at(&mut self, time_ms: TimeMS) {
        if let Some(trace_helper) = &mut self.trace_helper {
            trace_helper.read_data(time_ms);
        }
        if let Some(rsu_placement) = &mut self.rsu_placement {
            rsu_placement.read_data(time_ms);
        }
    }

    pub(crate) fn complete(self) {
        if let Some(trace_helper) = self.trace_helper {
            trace_helper.complete();
        }
        if let Some(rsu_placement) = self.rsu_placement {
            rsu_placement.complete();
        }
    }
}
