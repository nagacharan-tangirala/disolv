use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Debug, Copy, Clone, TypedBuilder)]
pub struct FlTrainingData {
    agent_id: u64,
    train_len: u32,
    test_len: u32,
}

#[derive(Debug)]
pub struct FlTrainingTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    train_data_len: Vec<u32>,
    test_data_len: Vec<u32>,
    to_output: WriterType,
}

impl FlTrainingTrace {
    pub(crate) fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: WriterType::new(output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            test_data_len: Vec::new(),
            train_data_len: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_ms: TimeMS, training_data: FlTrainingData) {
        self.time_step.push(time_ms.as_u64());
        self.agent_id.push(training_data.agent_id);
        self.train_data_len.push(training_data.train_len);
        self.test_data_len.push(training_data.test_len);
    }
}

impl ResultWriter for FlTrainingTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let train_len = Field::new("agent_id", DataType::UInt32, false);
        let test_len = Field::new("agent_id", DataType::UInt32, false);
        Schema::new(vec![time_ms, agent_id, train_len, test_len])
    }

    fn write_to_file(&mut self) {
        let record_batch = RecordBatch::try_from_iter(vec![
            (
                "time_step",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step))) as ArrayRef,
            ),
            (
                "agent_id",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.agent_id))) as ArrayRef,
            ),
            (
                "train_datasize",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.train_data_len))) as ArrayRef,
            ),
            (
                "test_datasize",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.test_data_len))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        self.to_output.record_batch_to_file(&record_batch);
    }

    fn close_file(self) {
        self.to_output.close()
    }
}
