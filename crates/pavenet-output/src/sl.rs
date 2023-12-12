use crate::result::{OutputSettings, OutputType};
use crate::rx::DataRx;
use crate::writer::DataOutput;
use log::debug;
use pavenet_core::message::RxMetrics;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::node::NodeId;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SlDataWriter {
    data_sl: Vec<DataRx>,
    to_output: DataOutput,
}

impl SlDataWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::SlData)
            .expect("SlDataWriter::new: No SlDataWriter config found");
        let output_file = output_path.join(&config.output_filename);
        debug!("SlDataWriter::new: output_file: {:?}", output_file);
        Self {
            data_sl: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, to_node: NodeId, rx_metrics: &RxMetrics) {
        let data_rx = DataRx::from_data(time_step, to_node, rx_metrics);
        self.data_sl.push(data_rx);
    }

    pub fn write_to_file(&mut self) {
        self.to_output.write_to_file(&self.data_sl);
        self.data_sl.clear();
    }
}
