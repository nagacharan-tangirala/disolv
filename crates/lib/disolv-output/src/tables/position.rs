use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_models::device::mobility::MapState;

use crate::result::ResultWriter;
use crate::writer::WriterType;

#[derive(Debug)]
pub struct PositionWriter {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    x: Vec<f64>,
    y: Vec<f64>,
    to_output: WriterType,
}

impl PositionWriter {
    pub fn new(output_file: PathBuf) -> Self {
        Self {
            to_output: WriterType::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            x: Vec::new(),
            y: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, agent_id: AgentId, map_state: &MapState) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(agent_id.as_u64());
        self.x.push(map_state.pos.x);
        self.y.push(map_state.pos.y);
    }
}

impl ResultWriter for PositionWriter {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let x = Field::new("x", DataType::Float64, false);
        let y = Field::new("distance", DataType::Float64, false);
        Schema::new(vec![time_ms, agent_id, x, y])
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
                "x",
                Arc::new(Float64Array::from(std::mem::take(&mut self.x))) as ArrayRef,
            ),
            (
                "y",
                Arc::new(Float64Array::from(std::mem::take(&mut self.y))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        self.to_output.record_batch_to_file(&record_batch);
    }

    fn close_file(self) {
        self.to_output.close()
    }
}
