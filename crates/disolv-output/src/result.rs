use crate::net::NetStatWriter;
use crate::position::PosWriter;
use crate::rx_counts::RxCountWriter;
use crate::tx::TxDataWriter;
use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_models::device::mobility::MapState;
use disolv_models::net::message::{DPayload, TxMetrics};
use disolv_models::net::radio::{DLink, OutgoingStats};
use disolv_models::net::slice::Slice;
use log::debug;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxCounts,
    TxData,
    NodePos,
    NetStat,
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
    pub output_interval: TimeMS,
    pub output_path: String,
    pub file_type: FileType,
    pub file_out_config: Vec<FileOutConfig>,
}

#[derive(Debug, Clone)]
pub struct ResultWriter {
    tx_writer: Option<TxDataWriter>,
    rx_count_writer: Option<RxCountWriter>,
    node_pos_writer: Option<PosWriter>,
    net_stat_writer: Option<NetStatWriter>,
}

impl ResultWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let tx_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::TxData)
            .map(|_| TxDataWriter::new(output_settings));
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
        let net_stat_writer = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::NetStat)
            .map(|_| NetStatWriter::new(output_settings));
        Self {
            tx_writer,
            rx_count_writer,
            node_pos_writer,
            net_stat_writer,
        }
    }

    pub fn add_rx_counts(
        &mut self,
        time_step: TimeMS,
        node_id: AgentId,
        in_data_stats: &OutgoingStats,
    ) {
        match &mut self.rx_count_writer {
            Some(rx) => {
                rx.add_data(time_step, node_id, in_data_stats);
            }
            None => (),
        }
    }

    pub fn add_tx_data(
        &mut self,
        time_step: TimeMS,
        link: &DLink,
        payload: &DPayload,
        tx_metrics: TxMetrics,
    ) {
        match &mut self.tx_writer {
            Some(tx) => {
                tx.add_data(time_step, link, payload, tx_metrics);
            }
            None => (),
        }
    }

    pub fn add_node_pos(&mut self, time_step: TimeMS, node_id: AgentId, map_state: &MapState) {
        match &mut self.node_pos_writer {
            Some(pos) => pos.add_data(time_step, node_id, map_state),
            None => (),
        }
    }

    pub fn add_net_stats(&mut self, time_step: TimeMS, slice: &Slice) {
        match &mut self.net_stat_writer {
            Some(net) => net.add_data(time_step, slice),
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
        match &mut self.net_stat_writer {
            Some(writer) => writer.write_to_file(),
            None => (),
        };
    }
}
