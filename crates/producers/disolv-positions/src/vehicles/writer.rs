use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use log::debug;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, TIME_STEP};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::produce::config::TraceSettings;

#[derive(Copy, Clone, Default, TypedBuilder, Deserialize)]
pub(crate) struct TraceInfo {
    pub(crate) time_ms: u64,
    pub(crate) agent_id: u64,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Display for TraceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "time_ms: {}, agent_id: {}, x: {}, y: {}",
            self.time_ms, self.agent_id, self.x, self.y
        )
    }
}

pub(crate) struct TraceCache {
    time_steps: Vec<u64>,
    agent_ids: Vec<u64>,
    x: Vec<f64>,
    y: Vec<f64>,
    cache_limit: usize,
}

impl TraceCache {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self {
            time_steps: Vec::with_capacity(cache_size),
            agent_ids: Vec::with_capacity(cache_size),
            x: Vec::with_capacity(cache_size),
            y: Vec::with_capacity(cache_size),
            cache_limit: (cache_size * 90) / 100,
        }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.time_steps.len() >= self.cache_limit
    }

    pub(crate) fn append_trace(&mut self, trace_info: TraceInfo) {
        self.time_steps.push(trace_info.time_ms);
        self.agent_ids.push(trace_info.agent_id);
        self.x.push(trace_info.x);
        self.y.push(trace_info.y);
    }

    pub(crate) fn as_record_batch(&mut self) -> RecordBatch {
        RecordBatch::try_from_iter(vec![
            (
                TIME_STEP,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.time_steps))) as ArrayRef,
            ),
            (
                AGENT_ID,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.agent_ids))) as ArrayRef,
            ),
            (
                COORD_X,
                Arc::new(Float64Array::from(std::mem::take(&mut self.x))) as ArrayRef,
            ),
            (
                COORD_Y,
                Arc::new(Float64Array::from(std::mem::take(&mut self.y))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert writer cache to record batch")
    }
}

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

    fn close_file(mut self) {
        debug!(r"Trace parsing is done. Flushing the cache to file");
        self.writer
            .record_batch_to_file(&self.trace_cache.as_record_batch());
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
}
