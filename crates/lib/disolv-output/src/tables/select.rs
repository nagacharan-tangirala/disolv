use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Copy, Clone, TypedBuilder)]
pub struct ClientSelectData {
    server_id: u64,
    available: u32,
    selected: u32,
}

#[derive(Debug)]
pub struct ClientSelectTrace {
    time_step: Vec<u64>,
    server_id: Vec<u64>,
    available: Vec<u32>,
    selected: Vec<u32>,
    to_output: WriterType,
}

impl ClientSelectTrace {
    pub(crate) fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: WriterType::new(output_file, Self::schema()),
            time_step: Vec::new(),
            server_id: Vec::new(),
            available: Vec::new(),
            selected: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_ms: TimeMS, selection_data: ClientSelectData) {
        self.time_step.push(time_ms.as_u64());
        self.server_id.push(selection_data.server_id);
        self.available.push(selection_data.available);
        self.selected.push(selection_data.selected);
    }
}

impl ResultWriter for ClientSelectTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let server_id = Field::new("server_id", DataType::UInt64, false);
        let available = Field::new("available", DataType::UInt32, false);
        let selected = Field::new("selected", DataType::UInt32, false);
        Schema::new(vec![time_ms, server_id, available, selected])
    }

    fn write_to_file(&mut self) {
        let record_batch = RecordBatch::try_from_iter(vec![
            (
                "time_step",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step))) as ArrayRef,
            ),
            (
                "agent_id",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.server_id))) as ArrayRef,
            ),
            (
                "available",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.available))) as ArrayRef,
            ),
            (
                "selected",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.selected))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        self.to_output.record_batch_to_file(&record_batch);
    }

    fn close_file(self) {
        self.to_output.close()
    }
}
