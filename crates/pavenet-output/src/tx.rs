use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use pavenet_core::message::DPayload;
use pavenet_core::radio::DLink;
use pavenet_engine::bucket::{Resultant, TimeMS};
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
    pub(crate) link_found: u32,
}

impl Resultant for DataTx {}

impl DataTx {
    fn from_data(time_step: TimeMS, link: &DLink, payload: &DPayload) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: payload.node_state.node_info.id.as_u32(),
            selected_node: link.target.as_u32(),
            distance: link.properties.distance.unwrap_or(-1.0),
            data_size: payload.metadata.total_size,
            data_count: payload.metadata.total_count,
            link_found: time_step.as_u32(),
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

    pub fn add_data(&mut self, time_step: TimeMS, link: &DLink, payload: &DPayload) {
        let data_tx = DataTx::from_data(time_step, link, payload);
        self.data_tx.push(data_tx);
    }

    pub fn write_to_file(&mut self) {
        self.to_output.write_to_file(&self.data_tx);
        self.data_tx.clear();
    }
}
