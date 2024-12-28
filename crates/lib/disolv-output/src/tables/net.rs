use std::path::PathBuf;
use std::sync::Arc;

use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::arrow::array::{ArrayRef, RecordBatch, UInt32Array, UInt64Array};
use crate::arrow::datatypes::{DataType, Field, Schema};
use crate::result::{ResultWriter, WriterType};

#[derive(Copy, Debug, Clone, TypedBuilder)]
pub struct NetStats {
    slice_id: u32,
    bandwidth: u64,
}

#[derive(Debug)]
pub struct NetStatWriter {
    time_step: Vec<u64>,
    slice_id: Vec<u32>,
    bandwidth: Vec<u64>,
    to_output: WriterType,
}

impl NetStatWriter {
    pub fn new(output_file: PathBuf) -> Self {
        Self {
            to_output: WriterType::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            slice_id: Vec::new(),
            bandwidth: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_ms: TimeMS, stats: NetStats) {
        self.time_step.push(time_ms.as_u64());
        self.slice_id.push(stats.slice_id);
        self.bandwidth.push(stats.bandwidth);
    }
}

impl ResultWriter for NetStatWriter {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let slice_id = Field::new("slice_id", DataType::UInt32, false);
        let bandwidth = Field::new("bandwidth", DataType::UInt64, false);
        Schema::new(vec![time_ms, slice_id, bandwidth])
    }

    fn write_to_file(&mut self) {
        let record_batch = RecordBatch::try_from_iter(vec![
            (
                "time_step",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step))) as ArrayRef,
            ),
            (
                "slice_id",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.slice_id))) as ArrayRef,
            ),
            (
                "bandwidth",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.bandwidth))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        match &mut self.to_output {
            WriterType::Parquet(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write parquet");
            }
            WriterType::Csv(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write csv");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            WriterType::Parquet(to_output) => to_output.close(),
            WriterType::Csv(to_output) => to_output.close(),
        }
    }
}
