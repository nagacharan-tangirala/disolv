use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use log::debug;
use pavenet_core::radio::InDataStats;
use pavenet_engine::bucket::{Resultant, TimeS};
use pavenet_engine::node::NodeId;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct DataRx {
    pub(crate) time_step: u32,
    pub(crate) node_id: u32,
    pub(crate) attempted_in_node_count: u32,
    pub(crate) attempted_in_data_size: f32,
    pub(crate) attempted_in_data_count: u32,
    pub(crate) feasible_in_node_count: u32,
    pub(crate) feasible_in_data_size: f32,
    pub(crate) feasible_in_data_count: u32,
}

impl DataRx {
    pub fn from_data(time_step: TimeS, node_id: NodeId, in_data_stats: &InDataStats) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: node_id.as_u32(),
            attempted_in_node_count: in_data_stats.attempted.node_count,
            attempted_in_data_size: in_data_stats.attempted.data_size,
            attempted_in_data_count: in_data_stats.attempted.data_count,
            feasible_in_node_count: in_data_stats.feasible.node_count,
            feasible_in_data_size: in_data_stats.feasible.data_size,
            feasible_in_data_count: in_data_stats.feasible.data_count,
        }
    }
}

impl Resultant for DataRx {}

#[derive(Debug, Clone)]
pub struct RxDataWriter {
    data_rx: Vec<DataRx>,
    to_output: DataOutput,
}

impl RxDataWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::RxData)
            .expect("RxDataWriter::new: No RxDataWriter config found");
        let output_file = output_path.join(&config.output_filename);
        debug!("RxDataWriter::new: output_file: {:?}", output_file);
        Self {
            data_rx: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeS, node_id: NodeId, in_data_stats: &InDataStats) {
        self.data_rx
            .push(DataRx::from_data(time_step, node_id, in_data_stats));
    }

    pub fn write_to_file(&mut self) {
        debug!("Writing tx data");
        self.to_output.write_to_file(&self.data_rx);
        self.data_rx.clear();
    }
}
