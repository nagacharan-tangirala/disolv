use std::mem::take;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, RecordBatch, StringArray, UInt32Array, UInt64Array};
use arrow::csv;
use arrow::datatypes::{DataType, Field, Schema};
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentId, AgentProperties};
use disolv_core::bucket::TimeMS;
use disolv_core::radio::Link;
use disolv_models::net::radio::LinkProperties;
use disolv_output::result::{BasicResults, OutputSettings, OutputType, ResultWriter};
use disolv_output::writer::DataOutput;

use crate::models::device::message::{FlPayload, TxMetrics};

#[derive(TypedBuilder)]
pub struct OutputWriter {
    pub basic_results: BasicResults,
    pub tx_data_writer: Option<TxDataWriter>,
    pub fl_state_writer: Option<StateTrace>,
    pub fl_model_writer: Option<ModelTrace>,
}

impl OutputWriter {
    pub fn new(output_config: &OutputSettings) -> Self {
        let basic_results = BasicResults::new(output_config);
        let output_path = PathBuf::from(&output_config.output_path);

        let tx_data_writer = output_config
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::TxData)
            .last()
            .map(|settings| TxDataWriter::new(&output_path.join(&settings.output_filename)));
        let fl_state_writer = output_config
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlState)
            .last()
            .map(|settings| StateTrace::new(&output_path.join(&settings.output_filename)));
        let fl_model_writer = output_config
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlModel)
            .last()
            .map(|settings| ModelTrace::new(&output_path.join(&settings.output_filename)));

        Self {
            basic_results,
            tx_data_writer,
            fl_state_writer,
            fl_model_writer,
        }
    }

    pub(crate) fn write_to_file(&mut self) {
        self.basic_results.positions.write_to_file();
        self.basic_results.rx_counts.write_to_file();
        if let Some(tx) = &mut self.tx_data_writer {
            tx.write_to_file();
        }
        if let Some(state) = &mut self.fl_state_writer {
            state.write_to_file();
        }
        if let Some(model) = &mut self.fl_model_writer {
            model.write_to_file();
        }
    }

    pub(crate) fn close_output_files(self) {
        self.basic_results.close_files();
        if let Some(tx) = self.tx_data_writer {
            tx.close_file();
        }
        if let Some(state) = self.fl_state_writer {
            state.close_file();
        }
        if let Some(model) = self.fl_model_writer {
            model.close_file();
        }
    }
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
    payload_size: Vec<u64>,
    to_output: DataOutput,
}

impl TxDataWriter {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: DataOutput::new(output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            selected_agent: Vec::new(),
            distance: Vec::new(),
            data_count: Vec::new(),
            link_found: Vec::new(),
            tx_order: Vec::new(),
            payload_size: Vec::new(),
        }
    }

    pub fn add_data(
        &mut self,
        time_step: TimeMS,
        link: &Link<LinkProperties>,
        payload: &FlPayload,
        tx_metrics: TxMetrics,
    ) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(payload.agent_state.id().as_u64());
        self.selected_agent.push(link.target.as_u64());
        self.distance.push(link.properties.distance.unwrap_or(-1.0));
        self.data_count.push(payload.metadata.total_count);
        self.link_found.push(time_step.as_u64());
        self.tx_order.push(tx_metrics.tx_order);
        self.payload_size.push(tx_metrics.payload_size.as_u64());
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
        let payload_size = Field::new("payload_size", DataType::UInt64, false);
        Schema::new(vec![
            time_ms,
            agent_id,
            selected_agent,
            distance,
            data_count,
            link_found,
            tx_order,
            payload_size,
        ])
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
                "selected_agent",
                Arc::new(UInt64Array::from(take(&mut self.selected_agent))) as ArrayRef,
            ),
            (
                "distance",
                Arc::new(Float32Array::from(take(&mut self.distance))) as ArrayRef,
            ),
            (
                "data_count",
                Arc::new(UInt32Array::from(take(&mut self.data_count))) as ArrayRef,
            ),
            (
                "link_found",
                Arc::new(UInt64Array::from(take(&mut self.link_found))) as ArrayRef,
            ),
            (
                "tx_order",
                Arc::new(UInt32Array::from(take(&mut self.tx_order))) as ArrayRef,
            ),
            (
                "payload_size",
                Arc::new(UInt64Array::from(take(&mut self.payload_size))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert results to record batch");
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write record batches to parquet");
            }
            DataOutput::Csv(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write record batches to CSV");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
            DataOutput::Csv(to_output) => to_output.close(),
        }
    }
}

#[derive(Debug)]
pub struct StateTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    state: Vec<String>,
    to_output: DataOutput,
}

impl StateTrace {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: DataOutput::new(output_file, Self::schema()),
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

impl ResultWriter for StateTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let states = Field::new("states", DataType::Utf8, false);
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
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write parquet");
            }
            DataOutput::Csv(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write csv");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
            DataOutput::Csv(to_output) => to_output.close(),
        }
    }
}

#[derive(Debug, TypedBuilder)]
pub(crate) struct ModelUpdate {
    time_step: TimeMS,
    agent_id: AgentId,
    target_id: AgentId,
    agent_state: String,
    model: String,
    direction: String,
    status: String,
    accuracy: f32,
}

#[derive(Debug)]
pub(crate) struct ModelTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    target_id: Vec<u64>,
    state: Vec<String>,
    model: Vec<String>,
    direction: Vec<String>,
    status: Vec<String>,
    accuracy: Vec<f32>,
    to_output: DataOutput,
}

impl ModelTrace {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: DataOutput::new(output_file, Self::schema()),
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
        self.time_step.push(model_update.time_step.as_u64());
        self.agent_id.push(model_update.agent_id.as_u64());
        self.target_id.push(model_update.target_id.as_u64());
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
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write parquet");
            }
            DataOutput::Csv(to_output) => {
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write csv");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
            DataOutput::Csv(to_output) => to_output.close(),
        }
    }
}
