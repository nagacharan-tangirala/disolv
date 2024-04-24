use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use arrow::array::{ArrayRef, Float32Array, RecordBatch, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_models::net::radio::OutgoingStats;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct RxCountWriter {
    time_step: Vec<u64>,
    agent_id: Vec<u64>,
    in_agent_count: Vec<u32>,
    in_data_size: Vec<u64>,
    in_data_count: Vec<u32>,
    to_output: DataOutput,
}

impl RxCountWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::RxCounts)
            .expect("RxDataWriter::new: No RxDataWriter config found");
        let output_file = output_path.join(&config.output_filename);
        Self {
            to_output: DataOutput::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            agent_id: Vec::new(),
            in_agent_count: Vec::new(),
            in_data_size: Vec::new(),
            in_data_count: Vec::new(),
        }
    }

    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let agent_id = Field::new("agent_id", DataType::UInt64, false);
        let in_agent_count = Field::new("in_agent_count", DataType::UInt32, false);
        let in_data_size = Field::new("in_data_size", DataType::UInt64, false);
        let in_data_count = Field::new("in_data_count", DataType::UInt32, false);
        Schema::new(vec![
            time_ms,
            agent_id,
            in_agent_count,
            in_data_size,
            in_data_count,
        ])
    }

    pub fn add_data(
        &mut self,
        time_step: TimeMS,
        agent_id: AgentId,
        in_data_stats: &OutgoingStats,
    ) {
        self.time_step.push(time_step.as_u64());
        self.agent_id.push(agent_id.as_u64());
        self.in_agent_count
            .push(in_data_stats.out_counts.agent_count);
        self.in_data_size
            .push(in_data_stats.out_counts.data_size.as_u64());
        self.in_data_count.push(in_data_stats.out_counts.data_count);
    }

    pub fn write_to_file(&mut self) {
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
                        "attempted_in_agent_count",
                        Arc::new(UInt32Array::from(std::mem::take(&mut self.in_agent_count)))
                            as ArrayRef,
                    ),
                    (
                        "attempted_in_data_size",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.in_data_size)))
                            as ArrayRef,
                    ),
                    (
                        "attempted_in_data_count",
                        Arc::new(UInt32Array::from(std::mem::take(&mut self.in_data_count)))
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

    pub(crate) fn close_files(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
        }
    }
}
