use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use advaitars_core::agent::AgentId;
use advaitars_core::bucket::{Resultant, TimeMS};
use advaitars_models::net::metrics::Bytes;
use advaitars_models::net::radio::OutgoingStats;
use log::debug;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct DataRxCounts {
    time_step: u32,
    node_id: u32,
    attempted_in_node_count: u32,
    attempted_in_data_size: Bytes,
    attempted_in_data_count: u32,
    feasible_in_node_count: u32,
    feasible_in_data_size: Bytes,
    feasible_in_data_count: u32,
    success_rate: f32,
}

impl DataRxCounts {
    pub fn from_data(time_step: TimeMS, node_id: AgentId, in_data_stats: &OutgoingStats) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: node_id.as_u32(),
            attempted_in_node_count: in_data_stats.attempted.node_count,
            attempted_in_data_size: in_data_stats.attempted.data_size,
            attempted_in_data_count: in_data_stats.attempted.data_count,
            feasible_in_node_count: in_data_stats.feasible.node_count,
            feasible_in_data_size: in_data_stats.feasible.data_size,
            feasible_in_data_count: in_data_stats.feasible.data_count,
            success_rate: in_data_stats.get_success_rate(),
        }
    }
}

impl Resultant for DataRxCounts {}

#[derive(Debug, Clone)]
pub struct RxCountWriter {
    data_rx: Vec<DataRxCounts>,
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
            data_rx: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, node_id: AgentId, in_data_stats: &OutgoingStats) {
        self.data_rx
            .push(DataRxCounts::from_data(time_step, node_id, in_data_stats));
    }

    pub fn write_to_file(&mut self) {
        debug!("Writing tx data");
        self.to_output
            .write_to_file(std::mem::take(&mut self.data_rx));
    }
}
