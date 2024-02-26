use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use advaitars_core::bucket::{Resultant, TimeMS};
use advaitars_models::net::message::{DPayload, TxFailReason, TxMetrics, TxStatus};
use advaitars_models::net::metrics::Bytes;
use advaitars_models::net::radio::DLink;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize)]
struct DataTx {
    pub(crate) time_step: u32,
    pub(crate) node_id: u32,
    pub(crate) selected_node: u32,
    pub(crate) distance: f32,
    pub(crate) data_count: u32,
    pub(crate) link_found: u32,
    pub(crate) tx_order: u32,
    pub(crate) tx_status: TxStatus,
    pub(crate) payload_size: Bytes,
    pub(crate) tx_fail_reason: TxFailReason,
    pub(crate) latency: u64,
}

impl Resultant for DataTx {}

impl DataTx {
    fn from_data(
        time_step: TimeMS,
        link: &DLink,
        tx_metrics: TxMetrics,
        payload: &DPayload,
    ) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: payload.node_state.device_info.id.as_u32(),
            selected_node: link.target.as_u32(),
            distance: link.properties.distance.unwrap_or(-1.0),
            data_count: payload.metadata.total_count,
            link_found: time_step.as_u32(),
            tx_order: tx_metrics.tx_order,
            tx_status: tx_metrics.tx_status,
            payload_size: tx_metrics.payload_size,
            tx_fail_reason: tx_metrics.tx_fail_reason,
            latency: tx_metrics.latency.as_u64(),
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

    pub fn add_data(
        &mut self,
        time_step: TimeMS,
        link: &DLink,
        payload: &DPayload,
        tx_metrics: TxMetrics,
    ) {
        let data_tx = DataTx::from_data(time_step, link, tx_metrics, payload);
        self.data_tx.push(data_tx);
    }

    pub fn write_to_file(&mut self) {
        self.to_output
            .write_to_file(std::mem::take(&mut self.data_tx));
    }
}
