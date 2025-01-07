use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use log::debug;
use typed_builder::TypedBuilder;

use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, TIME_STEP};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::produce::config::RSUSettings;

#[derive(Copy, Clone, TypedBuilder)]
pub struct RSUInfo {
    time_ms: u64,
    agent_id: u64,
    x: f64,
    y: f64,
}

pub(crate) struct RSUCache {
    time_steps: Vec<u64>,
    agent_ids: Vec<u64>,
    x: Vec<f64>,
    y: Vec<f64>,
    cache_limit: usize,
}

impl RSUCache {
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

    pub(crate) fn append_rsu(&mut self, rsu_info: RSUInfo) {
        self.time_steps.push(rsu_info.time_ms);
        self.agent_ids.push(rsu_info.agent_id);
        self.x.push(rsu_info.x);
        self.y.push(rsu_info.y);
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

pub(crate) struct RSUWriter {
    writer: WriterType,
    rsu_cache: RSUCache,
    _rsu_settings: RSUSettings,
}

impl ResultWriter for RSUWriter {
    fn schema() -> Schema {
        let time_ms = Field::new(TIME_STEP, DataType::UInt64, false);
        let agent_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let x = Field::new(COORD_X, DataType::Float64, false);
        let y = Field::new(COORD_Y, DataType::Float64, false);
        Schema::new(vec![time_ms, agent_id, x, y])
    }

    fn write_to_file(&mut self) {
        if self.rsu_cache.is_full() {
            debug!("RSU cache is full, writing");
            self.writer
                .record_batch_to_file(&self.rsu_cache.as_record_batch());
        }
    }

    fn close_file(mut self) {
        debug!(r"RSU parsing is done. Flushing the cache to file");
        self.writer
            .record_batch_to_file(&self.rsu_cache.as_record_batch());
        self.writer.close();
    }
}

impl RSUWriter {
    pub(crate) fn new(rsu_settings: &RSUSettings) -> Self {
        let rsu_file = PathBuf::from(rsu_settings.output_file.to_owned());
        let writer = WriterType::new(&rsu_file, Self::schema());
        let cache_size = 1000;
        Self {
            writer,
            rsu_cache: RSUCache::new(cache_size),
            _rsu_settings: rsu_settings.to_owned(),
        }
    }

    pub(crate) fn store_info(&mut self, rsu_info: RSUInfo) {
        self.rsu_cache.append_rsu(rsu_info);
    }
}
