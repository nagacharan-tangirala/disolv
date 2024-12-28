use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, RecordBatch, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_models::net::radio::OutgoingStats;

use crate::result::{ResultWriter, WriterType};

#[derive(Debug)]
pub struct RxCountWriter {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    attempted_in_agent_count: Vec<u32>,
    attempted_in_data_size: Vec<u64>,
    attempted_in_data_count: Vec<u32>,
    feasible_in_agent_count: Vec<u32>,
    feasible_in_data_size: Vec<u64>,
    feasible_in_data_count: Vec<u32>,
    success_rate: Vec<f32>,
    to_output: WriterType,
}

impl RxCountWriter {
    pub fn new(output_file: PathBuf) -> Self {
        Self {
            to_output: WriterType::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            attempted_in_agent_count: Vec::new(),
            attempted_in_data_size: Vec::new(),
            attempted_in_data_count: Vec::new(),
            feasible_in_agent_count: Vec::new(),
            feasible_in_data_size: Vec::new(),
            feasible_in_data_count: Vec::new(),
            success_rate: Vec::new(),
        }
    }

    pub fn add_data(
        &mut self,
        time_step: TimeMS,
        agent_id: AgentId,
        in_data_stats: &OutgoingStats,
    ) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(agent_id.as_u64());
        self.attempted_in_agent_count
            .push(in_data_stats.attempted.agent_count);
        self.attempted_in_data_size
            .push(in_data_stats.attempted.data_size.as_u64());
        self.attempted_in_data_count
            .push(in_data_stats.attempted.data_count);
        self.feasible_in_agent_count
            .push(in_data_stats.feasible.agent_count);
        self.feasible_in_data_size
            .push(in_data_stats.feasible.data_size.as_u64());
        self.feasible_in_data_count
            .push(in_data_stats.feasible.data_count);
        self.success_rate.push(in_data_stats.get_success_rate());
    }
}

impl ResultWriter for RxCountWriter {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let attempted_in_agent_count =
            Field::new("attempted_in_agent_count", DataType::UInt32, false);
        let attempted_in_data_size = Field::new("attempted_in_data_size", DataType::UInt64, false);
        let attempted_in_data_count =
            Field::new("attempted_in_data_count", DataType::UInt32, false);
        let feasible_in_agent_count =
            Field::new("feasible_in_agent_count", DataType::UInt32, false);
        let feasible_in_data_size = Field::new("feasible_in_data_size", DataType::UInt64, false);
        let feasible_in_data_count = Field::new("feasible_in_data_count", DataType::UInt32, false);
        let success_rate = Field::new("success_rate", DataType::Float32, false);
        Schema::new(vec![
            time_ms,
            agent_id,
            attempted_in_agent_count,
            attempted_in_data_size,
            attempted_in_data_count,
            feasible_in_agent_count,
            feasible_in_data_size,
            feasible_in_data_count,
            success_rate,
        ])
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
                "attempted_in_agent_count",
                Arc::new(UInt32Array::from(std::mem::take(
                    &mut self.attempted_in_agent_count,
                ))) as ArrayRef,
            ),
            (
                "attempted_in_data_size",
                Arc::new(UInt64Array::from(std::mem::take(
                    &mut self.attempted_in_data_size,
                ))) as ArrayRef,
            ),
            (
                "attempted_in_data_count",
                Arc::new(UInt32Array::from(std::mem::take(
                    &mut self.attempted_in_data_count,
                ))) as ArrayRef,
            ),
            (
                "feasible_in_agent_count",
                Arc::new(UInt32Array::from(std::mem::take(
                    &mut self.feasible_in_agent_count,
                ))) as ArrayRef,
            ),
            (
                "feasible_in_data_size",
                Arc::new(UInt64Array::from(std::mem::take(
                    &mut self.feasible_in_data_size,
                ))) as ArrayRef,
            ),
            (
                "feasible_in_data_count",
                Arc::new(UInt32Array::from(std::mem::take(
                    &mut self.feasible_in_data_count,
                ))) as ArrayRef,
            ),
            (
                "success_rate",
                Arc::new(Float32Array::from(std::mem::take(&mut self.success_rate))) as ArrayRef,
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
