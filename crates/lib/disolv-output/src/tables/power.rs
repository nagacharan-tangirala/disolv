use std::mem::take;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Debug)]
pub struct PowerTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    state: Vec<String>,
    to_output: WriterType,
}

impl PowerTrace {
    pub fn new(output_file: PathBuf) -> Self {
        Self {
            to_output: WriterType::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            state: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, agent_id: AgentId, state: String) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(agent_id.as_u64());
        self.state.push(state);
    }
}

impl ResultWriter for PowerTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let states = Field::new("state", DataType::Utf8, false);
        Schema::new(vec![time_ms, agent_id, states])
    }

    fn write_to_file(&mut self) {
        let record_batch = RecordBatch::try_from_iter(vec![
            (
                "time_step",
                Arc::new(UInt64Array::from(take(&mut self.time_step))) as ArrayRef,
            ),
            (
                "agent_id",
                Arc::new(UInt64Array::from(take(&mut self.agent_id))) as ArrayRef,
            ),
            (
                "state",
                Arc::new(StringArray::from(take(&mut self.state))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        self.to_output.record_batch_to_file(&record_batch);
    }

    fn close_file(self) {
        self.to_output.close()
    }
}
