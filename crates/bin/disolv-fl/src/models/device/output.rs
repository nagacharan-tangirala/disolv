use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, RecordBatch, UInt32Array, UInt64Array};
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
    pub fl_state_writer: Option<FlStateTrace>,
}

impl OutputWriter {
    pub fn new(output_config: &OutputSettings) -> Self {
        let basic_results = BasicResults::new(&output_config);
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
            .map(|settings| FlStateTrace::new(&output_path.join(&settings.output_filename)));

        Self {
            basic_results,
            tx_data_writer,
            fl_state_writer,
        }
    }

    pub(crate) fn write_to_file(&mut self) {
        self.basic_results.positions.write_to_file();
        self.basic_results.rx_counts.write_to_file();
        if let Some(tx) = &mut self.tx_data_writer {
            tx.write_to_file();
        }
        if let Some(fl) = &mut self.fl_state_writer {
            fl.write_to_file();
        }
    }

    pub fn close_output_files(mut self) {
        self.basic_results.close_files();
        if let Some(tx) = self.tx_data_writer {
            tx.close_file();
        }
        if let Some(fl) = self.fl_state_writer {
            fl.close_file();
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
            to_output: DataOutput::new(&output_file, Self::schema()),
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
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                let record_batch = RecordBatch::try_from_iter(vec![
                    (
                        "time_step",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step)))
                            as ArrayRef,
                    ),
                    (
                        "agent_id",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.agent_id))) as ArrayRef,
                    ),
                    (
                        "selected_agent",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.selected_agent)))
                            as ArrayRef,
                    ),
                    (
                        "distance",
                        Arc::new(Float32Array::from(std::mem::take(&mut self.distance)))
                            as ArrayRef,
                    ),
                    (
                        "data_count",
                        Arc::new(UInt32Array::from(std::mem::take(&mut self.data_count)))
                            as ArrayRef,
                    ),
                    (
                        "link_found",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.link_found)))
                            as ArrayRef,
                    ),
                    (
                        "tx_order",
                        Arc::new(UInt32Array::from(std::mem::take(&mut self.tx_order))) as ArrayRef,
                    ),
                    (
                        "payload_size",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.payload_size)))
                            as ArrayRef,
                    ),
                ])
                .expect("Failed to convert results to record batch");
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write record batches to file");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
        }
    }
}

#[derive(Debug)]
pub struct FlStateTrace {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    states: Vec<u64>,
    to_output: DataOutput,
}

impl FlStateTrace {
    pub fn new(output_file: &PathBuf) -> Self {
        Self {
            to_output: DataOutput::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, agent_id: AgentId, state: u64) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(agent_id.as_u64());
        self.states.push(state);
    }
}

impl ResultWriter for FlStateTrace {
    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let states = Field::new("states", DataType::UInt64, false);
        Schema::new(vec![time_ms, agent_id, states])
    }

    fn write_to_file(&mut self) {
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                let record_batch = RecordBatch::try_from_iter(vec![
                    (
                        "time_step",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step)))
                            as ArrayRef,
                    ),
                    (
                        "agent_id",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.agent_id))) as ArrayRef,
                    ),
                    (
                        "states",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.states))) as ArrayRef,
                    ),
                ])
                .expect("Failed to convert results to record batch");
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write record batches to file");
            }
        }
    }

    fn close_file(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
        }
    }
}
