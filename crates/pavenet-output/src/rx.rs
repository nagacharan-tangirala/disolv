use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use pavenet_core::message::{RxFailReason, RxMetrics, RxStatus};
use pavenet_engine::bucket::{Resultant, TimeMS};
use pavenet_engine::node::NodeId;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct DataRx {
    time_step: u32,
    from: u32,
    to: u32,
    rx_order: u32,
    rx_status: RxStatus,
    rx_fail_reason: RxFailReason,
    latency: u32,
}

impl DataRx {
    fn from_data(time_step: TimeMS, to_node: NodeId, rx_metrics: &RxMetrics) -> Self {
        Self {
            time_step: time_step.as_u32(),
            to: to_node.as_u32(),
            from: rx_metrics.from_node.as_u32(),
            rx_order: rx_metrics.rx_order,
            rx_status: rx_metrics.rx_status,
            rx_fail_reason: rx_metrics.rx_fail_reason,
            latency: rx_metrics.latency.as_u32(),
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
        Self {
            data_rx: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, to_node: NodeId, rx_metrics: &RxMetrics) {
        let data_rx = DataRx::from_data(time_step, to_node, rx_metrics);
        self.data_rx.push(data_rx);
    }

    pub fn write_to_file(&mut self) {
        self.to_output.write_to_file(&self.data_rx);
        self.data_rx.clear();
    }
}
