use crate::position::PosWriter;
use crate::rx::RxDataWriter;
use crate::rx_counts::RxCountWriter;
use crate::sl::SlDataWriter;
use crate::tx::TxDataWriter;
use log::debug;
use pavenet_core::message::{DPayload, RxMetrics};
use pavenet_core::mobility::MapState;
use pavenet_core::radio::{DLink, InDataStats};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::node::NodeId;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxCounts,
    RxData,
    TxData,
    NodePos,
    SlData,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum FileType {
    Csv,
    Parquet,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileOutConfig {
    pub output_type: OutputType,
    pub output_filename: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_step: TimeMS,
    pub output_path: String,
    pub file_type: FileType,
    pub file_out_config: Vec<FileOutConfig>,
}

#[derive(Debug, Clone)]
pub struct ResultWriter {
    tx_writer: Option<TxDataWriter>,
    rx_writer: Option<RxDataWriter>,
    sl_writer: Option<SlDataWriter>,
    rx_count_writer: Option<RxCountWriter>,
    node_pos_writer: Option<PosWriter>,
}

impl ResultWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let tx_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::TxData)
            .map(|_| TxDataWriter::new(output_settings));
        let rx_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::RxData)
            .map(|_| RxDataWriter::new(output_settings));
        let sl_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::SlData)
            .map(|_| SlDataWriter::new(output_settings));
        let node_pos_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::NodePos)
            .map(|_| PosWriter::new(output_settings));
        let rx_count_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::RxCounts)
            .map(|_| RxCountWriter::new(output_settings));
        Self {
            tx_writer,
            rx_count_writer,
            rx_writer,
            sl_writer,
            node_pos_writer,
        }
    }

    pub fn add_tx_data(&mut self, time_step: TimeMS, link: &DLink, payload: &DPayload) {
        match &mut self.tx_writer {
            Some(tx) => {
                tx.add_data(time_step, link, payload);
            }
            None => (),
        }
    }

    pub fn add_rx_counts(
        &mut self,
        time_step: TimeMS,
        node_id: NodeId,
        in_data_stats: &InDataStats,
    ) {
        match &mut self.rx_count_writer {
            Some(rx) => {
                rx.add_data(time_step, node_id, in_data_stats);
            }
            None => (),
        }
    }

    pub fn add_rx_data(&mut self, time_step: TimeMS, to_node: NodeId, rx_metrics: &RxMetrics) {
        match &mut self.rx_writer {
            Some(rx) => {
                rx.add_data(time_step, to_node, rx_metrics);
            }
            None => (),
        }
    }

    pub fn add_sl_data(&mut self, time_step: TimeMS, to_node: NodeId, rx_metrics: &RxMetrics) {
        match &mut self.sl_writer {
            Some(sl) => {
                sl.add_data(time_step, to_node, rx_metrics);
            }
            None => (),
        }
    }

    pub fn add_node_pos(&mut self, time_step: TimeMS, node_id: NodeId, map_state: &MapState) {
        match &mut self.node_pos_writer {
            Some(pos) => pos.add_data(time_step, node_id, map_state),
            None => (),
        }
    }

    pub fn write_output(&mut self, step: TimeMS) {
        debug!("Writing output at step {}", step.as_u32());
        match &mut self.tx_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
        match &mut self.rx_count_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
        match &mut self.node_pos_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
        match &mut self.rx_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
        match &mut self.sl_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
    }
}
