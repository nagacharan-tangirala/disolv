use std::mem::take;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Debug, TypedBuilder)]
pub struct PayloadUpdate {
    time_step: TimeMS,
    source: AgentId,
    target: AgentId,
    agent_state: String,
    payload_type: String,
    fl_content: String,
    action_type: String,
}

#[derive(Debug)]
pub struct PayloadTraceWriter {
    time_step: Vec<u64>,
    source: Vec<u64>,
    target: Vec<u64>,
    agent_state: Vec<String>,
    payload_type: Vec<String>,
    fl_content: Vec<String>,
    action_type: Vec<String>,
    to_output: WriterType,
}

impl PayloadTraceWriter {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: WriterType::new(output_file, Self::schema()),
            time_step: Vec::new(),
            source: Vec::new(),
            target: Vec::new(),
            agent_state: Vec::new(),
            payload_type: Vec::new(),
            fl_content: Vec::new(),
            action_type: Vec::new(),
        }
    }

    pub fn add_data(&mut self, payload_update: PayloadUpdate) {
        self.time_step.push(payload_update.time_step.as_u64());
        self.source.push(payload_update.source.as_u64());
        self.target.push(payload_update.target.as_u64());
        self.agent_state.push(payload_update.agent_state);
        self.payload_type.push(payload_update.payload_type);
        self.fl_content.push(payload_update.fl_content);
        self.action_type.push(payload_update.action_type);
    }
}

impl ResultWriter for PayloadTraceWriter {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let source = Field::new("source", DataType::UInt64, false);
        let target = Field::new("target", DataType::UInt64, false);
        let states = Field::new("agent_state", DataType::Utf8, false);
        let model = Field::new("payload_type", DataType::Utf8, false);
        let direction = Field::new("fl_content", DataType::Utf8, false);
        let status = Field::new("action_type", DataType::Utf8, false);
        Schema::new(vec![
            time_ms, source, target, states, model, direction, status,
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
                Arc::new(UInt64Array::from(take(&mut self.source))) as ArrayRef,
            ),
            (
                "target",
                Arc::new(UInt64Array::from(take(&mut self.target))) as ArrayRef,
            ),
            (
                "agent_state",
                Arc::new(StringArray::from(take(&mut self.agent_state))) as ArrayRef,
            ),
            (
                "payload_type",
                Arc::new(StringArray::from(take(&mut self.payload_type))) as ArrayRef,
            ),
            (
                "fl_content",
                Arc::new(StringArray::from(take(&mut self.fl_content))) as ArrayRef,
            ),
            (
                "action_type",
                Arc::new(StringArray::from(take(&mut self.action_type))) as ArrayRef,
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
