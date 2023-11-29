use crate::headers::data_rx::DataRx;
use crate::headers::node_pos::NodePosition;
use crate::headers::payload_tx::DataTx;
use crate::writer::FileOutput;
use log::debug;
use pavenet_core::mobility::MapState;
use pavenet_core::payload::DPayload;
use pavenet_core::radio::stats::InDataStats;
use pavenet_core::result::{OutputSettings, OutputType};
use pavenet_engine::bucket::TimeS;
use pavenet_engine::entity::NodeId;
use pavenet_engine::result::GResultWriter;
use std::path::PathBuf;

pub type DRxDataWriter = GResultWriter<DataRx, FileOutput>;
pub type DTxDataWriter = GResultWriter<DataTx, FileOutput>;
pub type DNodePosWriter = GResultWriter<NodePosition, FileOutput>;

#[derive(Debug, Clone)]
pub struct ResultWriter {
    pub rx_writer: Option<DRxDataWriter>,
    pub tx_writer: Option<DTxDataWriter>,
    pub node_pos_writer: Option<DNodePosWriter>,
    pub tx_data: Vec<DataTx>,
    pub rx_data: Vec<DataRx>,
    pub node_pos: Vec<NodePosition>,
}

impl ResultWriter {
    pub fn builder(
        output_path: &PathBuf,
        step_size: TimeS,
        output_settings: &OutputSettings,
    ) -> ResultWriterBuilder {
        ResultWriterBuilder {
            output_settings: output_settings.clone(),
            output_path: output_path.clone(),
            tx_writer: None,
            rx_writer: None,
            node_pos_writer: None,
            output_interval: output_settings.output_step,
            step_size,
        }
    }

    pub fn add_tx_data(&mut self, time_step: TimeS, payload: &DPayload) {
        match &mut self.tx_writer {
            Some(_) => {
                let data_tx = DataTx::from_data(time_step, payload);
                self.tx_data.push(data_tx);
            }
            None => (),
        }
    }

    pub fn add_rx_data(&mut self, time_step: TimeS, node_id: NodeId, in_data_stats: &InDataStats) {
        match &mut self.rx_writer {
            Some(_) => {
                let data_rx = DataRx::from_data(time_step, node_id, in_data_stats);
                self.rx_data.push(data_rx);
            }
            None => (),
        }
    }

    pub fn add_node_pos(&mut self, time_step: TimeS, node_id: NodeId, map_state: &MapState) {
        match &mut self.node_pos_writer {
            Some(_) => {
                let node_pos = NodePosition::from_data(time_step, node_id, map_state);
                self.node_pos.push(node_pos);
            }
            None => (),
        }
    }

    pub fn write_output(&mut self, step: TimeS) {
        debug!("Writing output at step {}", step.as_u32());
        self.write_tx_data(self.tx_data.to_owned());
        self.write_rx_data(self.rx_data.to_owned());
        self.write_node_pos(self.node_pos.to_owned());
        self.tx_data.clear();
        self.rx_data.clear();
        self.node_pos.clear();
    }

    fn write_tx_data(&mut self, data_tx: Vec<DataTx>) {
        match &mut self.tx_writer {
            Some(writer) => writer.write_results(data_tx),
            None => (),
        }
    }

    fn write_rx_data(&mut self, data_rx: Vec<DataRx>) {
        match &mut self.rx_writer {
            Some(writer) => writer.write_results(data_rx),
            None => (),
        }
    }

    fn write_node_pos(&mut self, node_pos: Vec<NodePosition>) {
        debug!("Writing node positions of size {}", node_pos.len());
        match &mut self.node_pos_writer {
            Some(writer) => writer.write_results(node_pos),
            None => (),
        }
    }
}

pub struct ResultWriterBuilder {
    pub output_path: PathBuf,
    pub output_settings: OutputSettings,
    pub tx_writer: Option<DTxDataWriter>,
    pub rx_writer: Option<DRxDataWriter>,
    pub node_pos_writer: Option<DNodePosWriter>,
    pub output_interval: TimeS,
    pub step_size: TimeS,
}

impl ResultWriterBuilder {
    pub fn build(self) -> ResultWriter {
        let vec_size = (self.output_interval.as_u32() / self.step_size.as_u32()) as usize;
        ResultWriter {
            rx_writer: self.rx_writer,
            tx_writer: self.tx_writer,
            node_pos_writer: self.node_pos_writer,
            tx_data: Vec::with_capacity(vec_size),
            rx_data: Vec::with_capacity(vec_size),
            node_pos: Vec::with_capacity(vec_size),
        }
    }

    pub fn with_rx_writer(mut self) -> Self {
        let rx_config = self
            .output_settings
            .file_out_config
            .iter()
            .find(|config| config.output_type == OutputType::RxData);
        self.rx_writer = match rx_config {
            Some(config) => {
                let file_path =
                    PathBuf::from(self.output_path.clone()).join(&config.output_filename);
                let writer = FileOutput::new(&file_path);
                let rx_writer = DRxDataWriter::new(writer);
                Some(rx_writer)
            }
            None => None,
        };
        self
    }

    pub fn with_tx_writer(mut self) -> Self {
        let tx_config = self
            .output_settings
            .file_out_config
            .iter()
            .find(|config| config.output_type == OutputType::TxData);
        self.tx_writer = match tx_config {
            Some(config) => {
                let file_path =
                    PathBuf::from(self.output_path.clone()).join(&config.output_filename);
                let writer = FileOutput::new(&file_path);
                let tx_writer = DTxDataWriter::new(writer);
                Some(tx_writer)
            }
            None => None,
        };
        self
    }

    pub fn with_node_pos_writer(mut self) -> Self {
        let node_pos_config = self
            .output_settings
            .file_out_config
            .iter()
            .find(|config| config.output_type == OutputType::NodePos);
        self.node_pos_writer = match node_pos_config {
            Some(config) => {
                let file_path =
                    PathBuf::from(self.output_path.clone()).join(&config.output_filename);
                let writer = FileOutput::new(&file_path);
                let node_pos_writer = DNodePosWriter::new(writer);
                Some(node_pos_writer)
            }
            None => None,
        };
        self
    }
}
