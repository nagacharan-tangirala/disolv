use std::path::PathBuf;
use std::sync::Arc;

use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::arrow::array::{ArrayRef, Float32Array, RecordBatch, UInt32Array, UInt64Array};
use crate::arrow::datatypes::{DataType, Field, Schema};
use crate::result::{ResultWriter, WriterType};

#[derive(TypedBuilder, Copy, Clone)]
pub struct TxData {
    agent_id: u64,
    selected_agent: u64,
    distance: f32,
    data_count: u32,
    link_found: u64,
    tx_order: u32,
    tx_status: u32,
    payload_size: u64,
    tx_fail_reason: u32,
    latency: u64,
}

#[derive(Debug)]
pub struct TxDataWriter {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    selected_agent: Vec<u64>,
    distance: Vec<f32>,
    data_count: Vec<u32>,
    link_found: Vec<u64>,
    tx_order: Vec<u32>,
    tx_status: Vec<u32>,
    payload_size: Vec<u64>,
    tx_fail_reason: Vec<u32>,
    latency: Vec<u64>,
    to_output: WriterType,
}

impl TxDataWriter {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: WriterType::new(output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            selected_agent: Vec::new(),
            distance: Vec::new(),
            data_count: Vec::new(),
            link_found: Vec::new(),
            tx_order: Vec::new(),
            tx_status: Vec::new(),
            payload_size: Vec::new(),
            tx_fail_reason: Vec::new(),
            latency: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, tx_data: TxData) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(tx_data.agent_id);
        self.selected_agent.push(tx_data.selected_agent);
        self.distance.push(tx_data.distance);
        self.data_count.push(tx_data.data_count);
        self.link_found.push(tx_data.link_found);
        self.tx_order.push(tx_data.tx_order);
        self.tx_status.push(tx_data.tx_status);
        self.payload_size.push(tx_data.payload_size);
        self.tx_fail_reason.push(tx_data.tx_fail_reason);
        self.latency.push(tx_data.latency);
    }
}
impl ResultWriter for TxDataWriter {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let selected_agent = Field::new("selected_agent", DataType::UInt64, false);
        let distance = Field::new("distance", DataType::Float32, false);
        let data_count = Field::new("data_count", DataType::UInt32, false);
        let link_found = Field::new("link_found", DataType::UInt64, false);
        let tx_order = Field::new("tx_order", DataType::UInt32, false);
        let tx_status = Field::new("tx_status", DataType::UInt32, false);
        let payload_size = Field::new("payload_size", DataType::UInt64, false);
        let tx_fail_reason = Field::new("tx_fail_reason", DataType::UInt32, false);
        let latency = Field::new("latency", DataType::UInt64, false);
        Schema::new(vec![
            time_ms,
            agent_id,
            selected_agent,
            distance,
            data_count,
            link_found,
            tx_order,
            tx_status,
            payload_size,
            tx_fail_reason,
            latency,
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
                "selected_agent",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.selected_agent))) as ArrayRef,
            ),
            (
                "distance",
                Arc::new(Float32Array::from(std::mem::take(&mut self.distance))) as ArrayRef,
            ),
            (
                "data_count",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.data_count))) as ArrayRef,
            ),
            (
                "link_found",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.link_found))) as ArrayRef,
            ),
            (
                "tx_order",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.tx_order))) as ArrayRef,
            ),
            (
                "tx_status",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.tx_status))) as ArrayRef,
            ),
            (
                "payload_size",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.payload_size))) as ArrayRef,
            ),
            (
                "tx_fail_reason",
                Arc::new(UInt32Array::from(std::mem::take(&mut self.tx_fail_reason))) as ArrayRef,
            ),
            (
                "latency",
                Arc::new(UInt64Array::from(std::mem::take(&mut self.latency))) as ArrayRef,
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
