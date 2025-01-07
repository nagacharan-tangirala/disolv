use std::mem::take;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, RecordBatch, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use typed_builder::TypedBuilder;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Debug, TypedBuilder)]
pub struct ModelUpdate {
    time_step: u64,
    agent_id: u64,
    target_id: u64,
    agent_state: String,
    model: String,
    direction: String,
    status: String,
    accuracy: f32,
}

#[derive(Debug)]
pub struct ModelTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    target_id: Vec<u64>,
    state: Vec<String>,
    model: Vec<String>,
    direction: Vec<String>,
    status: Vec<String>,
    accuracy: Vec<f32>,
    to_output: WriterType,
}

impl ModelTrace {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: WriterType::new(output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            target_id: Vec::new(),
            state: Vec::new(),
            model: Vec::new(),
            direction: Vec::new(),
            status: Vec::new(),
            accuracy: Vec::new(),
        }
    }

    pub fn add_data(&mut self, model_update: ModelUpdate) {
        self.time_step.push(model_update.time_step);
        self.agent_id.push(model_update.agent_id);
        self.target_id.push(model_update.target_id);
        self.state.push(model_update.agent_state);
        self.model.push(model_update.model);
        self.direction.push(model_update.direction);
        self.status.push(model_update.status);
        self.accuracy.push(model_update.accuracy);
    }
}

impl ResultWriter for ModelTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("source_id", DataType::UInt64, false);
        let target_id = Field::new("target_id", DataType::UInt64, false);
        let states = Field::new("states", DataType::Utf8, false);
        let model = Field::new("model", DataType::Utf8, false);
        let direction = Field::new("direction", DataType::Utf8, false);
        let status = Field::new("status", DataType::Utf8, false);
        let accuracy = Field::new("accuracy", DataType::Float32, false);
        Schema::new(vec![
            time_ms, agent_id, target_id, states, model, direction, status, accuracy,
        ])
    }

    fn write_to_file(&mut self) {
        let record_batch = RecordBatch::try_from_iter(vec![
            (
                "time_step",
                Arc::new(UInt64Array::from(take(&mut self.time_step))) as ArrayRef,
            ),
            (
                "source",
                Arc::new(UInt64Array::from(take(&mut self.agent_id))) as ArrayRef,
            ),
            (
                "target",
                Arc::new(UInt64Array::from(take(&mut self.target_id))) as ArrayRef,
            ),
            (
                "state",
                Arc::new(StringArray::from(take(&mut self.state))) as ArrayRef,
            ),
            (
                "model",
                Arc::new(StringArray::from(take(&mut self.model))) as ArrayRef,
            ),
            (
                "direction",
                Arc::new(StringArray::from(take(&mut self.direction))) as ArrayRef,
            ),
            (
                "status",
                Arc::new(StringArray::from(take(&mut self.status))) as ArrayRef,
            ),
            (
                "accuracy",
                Arc::new(Float32Array::from(take(&mut self.accuracy))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        self.to_output.record_batch_to_file(&record_batch);
    }

    fn close_file(self) {
        match self.to_output {
            WriterType::Parquet(to_output) => to_output.close(),
            WriterType::Csv(to_output) => to_output.close(),
        }
    }
}
