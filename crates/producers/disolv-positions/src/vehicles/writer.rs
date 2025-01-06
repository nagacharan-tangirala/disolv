use std::path::PathBuf;

use arrow::datatypes::{DataType, Field, Schema};
use log::debug;

use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, TIME_STEP};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::produce::config::TraceSettings;
use crate::trace::cache::{TraceCache, TraceInfo};

pub(crate) struct TraceWriter {
    writer: WriterType,
    trace_cache: TraceCache,
    _trace_settings: TraceSettings,
}

impl ResultWriter for TraceWriter {
    fn schema() -> Schema {
        let time_ms = Field::new(TIME_STEP, DataType::UInt64, false);
        let agent_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let x = Field::new(COORD_X, DataType::Float64, false);
        let y = Field::new(COORD_Y, DataType::Float64, false);
        Schema::new(vec![time_ms, agent_id, x, y])
    }

    fn write_to_file(&mut self) {
        if self.trace_cache.is_full() {
            debug!("Trace cache is full, writing");
            self.writer
                .record_batch_to_file(&self.trace_cache.as_record_batch());
        }
    }

    fn close_file(self) {
        self.writer.close();
    }
}

impl TraceWriter {
    pub(crate) fn new(trace_settings: &TraceSettings) -> Self {
        let trace_file = PathBuf::from(trace_settings.output_trace.to_owned());
        let writer = WriterType::new(&trace_file, Self::schema());
        let cache_size = 100000;
        Self {
            writer,
            trace_cache: TraceCache::new(cache_size),
            _trace_settings: trace_settings.to_owned(),
        }
    }

    pub(crate) fn store_info(&mut self, trace_info: TraceInfo) {
        self.trace_cache.append_trace(trace_info);
    }

    pub(crate) fn flush(mut self) {
        debug!(r"Trace parsing is done. Flushing the cache to file");
        self.writer
            .record_batch_to_file(&self.trace_cache.as_record_batch());
        self.writer.close();
    }
}
