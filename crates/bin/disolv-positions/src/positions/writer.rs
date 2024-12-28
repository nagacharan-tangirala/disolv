use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use log::debug;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;
use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, DISTANCE, TARGET_ID, TIME_STEP};

use crate::simulation::config::OutputSettings;

pub(crate) struct TraceWriter {
    writer: ArrowWriter<File>,
    writer_cache: WriterCache,
}

impl TraceWriter {
    pub(crate) fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(output_settings.output_path.to_owned());
        let output_file = output_path.join(output_settings.output_file.to_owned());
        let writer = Self::create_writer(output_file.to_str().expect("invalid output file"));
        let cache_size = 100000;
        Self {
            writer,
            writer_cache: WriterCache::new(cache_size),
        }
    }

    fn create_writer(output_file: &str) -> ArrowWriter<File> {
        debug!("Creating traces file {}", output_file);
        let time_ms = Field::new(TIME_STEP, DataType::UInt64, false);
        let source_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let x = Field::new(COORD_X, DataType::Float64, false);
        let y = Field::new(COORD_Y, DataType::Float64, false);

        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let schema = Schema::new(vec![time_ms, source_id, x, y]);
        let output_file = match File::create(output_file) {
            Ok(file) => file,
            Err(_) => panic!("Failed to create trace file to write"),
        };
        match ArrowWriter::try_new(output_file, SchemaRef::from(schema), Some(props)) {
            Ok(writer) => writer,
            Err(_) => panic!("Failed to create trace file writer"),
        }
    }

    pub(crate) fn store_info(&mut self, trace_info: TraceInfo) {
        self.writer_cache.append_trace(trace_info);
    }

    pub(crate) fn flush(mut self) {
        debug!(r"Trace parsing is done. Flushing the cache to file");
        self.writer
            .write(&self.writer_cache.as_record_batch())
            .expect("Failed to flush trace cache");
        self.writer.close().expect("Failed to close the trace file");
    }
}

#[derive(Copy, Clone, TypedBuilder)]
pub struct TraceInfo {
    time_ms: u64,
    agent_id: u64,
    x: f64,
    y: f64,
}

struct WriterCache {
    time_steps: Vec<u64>,
    agent_ids: Vec<u64>,
    x: Vec<f64>,
    y: Vec<f64>,
    cache_size: usize,
}

impl WriterCache {
    fn new(cache_size: usize) -> Self {
        Self {
            time_steps: Vec::with_capacity(cache_size),
            agent_ids: Vec::with_capacity(cache_size),
            x: Vec::with_capacity(cache_size),
            y: Vec::with_capacity(cache_size),
            cache_size,
        }
    }

    fn is_full(&self) -> bool {
        self.time_steps.len() >= self.cache_size
    }

    fn append_trace(&mut self, trace_info: TraceInfo) {
        self.time_steps.push(trace_info.time_ms);
        self.agent_ids.push(trace_info.agent_id);
        self.x.push(trace_info.x);
        self.y.push(trace_info.y);
    }

    fn as_record_batch(&mut self) -> RecordBatch {
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
