use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use log::debug;
use pavenet_core::message::DPayload;
use pavenet_engine::bucket::{Resultant, TimeS};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize)]
struct DataTx {
    pub(crate) time_step: u32,
    pub(crate) node_id: u32,
    pub(crate) selected_node: u32,
    pub(crate) distance: f32,
    pub(crate) data_size: f32,
    pub(crate) data_count: u32,
}

impl Resultant for DataTx {}

impl DataTx {
    fn from_data(time_step: TimeS, payload: &DPayload) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: payload.node_state.node_info.id.as_u32(),
            selected_node: payload.metadata.routing_info.selected_link.target.as_u32(),
            distance: payload
                .metadata
                .routing_info
                .selected_link
                .properties
                .distance
                .unwrap_or(-1.0),
            data_size: payload.metadata.total_size,
            data_count: payload.metadata.total_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TxDataWriter {
    data_tx: Vec<DataTx>,
    to_output: DataOutput,
}

impl TxDataWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::TxData)
            .expect("TxDataWriter::new: No TxDataWriter config found");
        let output_file = output_path.join(&config.output_filename);
        Self {
            data_tx: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeS, payload: &DPayload) {
        self.data_tx.push(DataTx::from_data(time_step, payload));
    }

    pub fn write_to_file(&mut self) {
        debug!("Writing tx data");
        self.to_output.write_to_file(&self.data_tx);
        self.data_tx.clear();
    }
}
