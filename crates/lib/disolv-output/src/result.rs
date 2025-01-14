use std::fs;
use std::path::Path;

use arrow::datatypes::Schema;
use serde::Deserialize;

use disolv_core::bucket::TimeMS;

use crate::tables::model::ModelTrace;
use crate::tables::net::NetStatWriter;
use crate::tables::payload::PayloadTraceWriter;
use crate::tables::position::PositionWriter;
use crate::tables::rx::RxCountWriter;
use crate::tables::select::ClientSelectTrace;
use crate::tables::state::StateTrace;
use crate::tables::train::FlTrainingTrace;
use crate::tables::tx::TxDataWriter;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxCounts,
    TxStats,
    AgentPos,
    NetStat,
    FlState,
    FlModel,
    FlPayloadTx,
    FlTraining,
    FlSelection,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_interval: TimeMS,
    pub output_path: String,
    pub outputs: Vec<Outputs>,
    pub scenario_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Outputs {
    pub output_type: OutputType,
    pub output_filename: String,
}

pub trait ResultWriter {
    fn schema() -> Schema;
    fn write_to_file(&mut self);
    fn close_file(self);
}
#[derive(Debug)]
pub struct Results {
    pub positions: Option<PositionWriter>,
    pub rx_counts: Option<RxCountWriter>,
    pub net_stats: Option<NetStatWriter>,
    pub tx_data: Option<TxDataWriter>,
    pub payload_tx: Option<PayloadTraceWriter>,
    pub model: Option<ModelTrace>,
    pub state: Option<StateTrace>,
    pub train: Option<FlTrainingTrace>,
    pub select: Option<ClientSelectTrace>,
}

impl Results {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = Path::new(&output_settings.output_path)
            .join(&output_settings.scenario_id)
            .join("files");
        if !output_path.exists() {
            fs::create_dir_all(&output_path).expect("Failed to create output directory");
        }

        let positions = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::AgentPos)
            .last()
            .map(|settings| PositionWriter::new(output_path.join(&settings.output_filename)));
        let rx_counts = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::RxCounts)
            .last()
            .map(|settings| RxCountWriter::new(output_path.join(&settings.output_filename)));
        let net_stats = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::NetStat)
            .last()
            .map(|settings| NetStatWriter::new(output_path.join(&settings.output_filename)));
        let tx_data = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::TxStats)
            .last()
            .map(|settings| TxDataWriter::new(&output_path.join(&settings.output_filename)));
        let payload_tx = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlPayloadTx)
            .last()
            .map(|settings| PayloadTraceWriter::new(&output_path.join(&settings.output_filename)));
        let model = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlModel)
            .last()
            .map(|settings| ModelTrace::new(&output_path.join(&settings.output_filename)));
        let state = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlState)
            .last()
            .map(|settings| StateTrace::new(&output_path.join(&settings.output_filename)));
        let train = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlTraining)
            .last()
            .map(|settings| FlTrainingTrace::new(&output_path.join(&settings.output_filename)));
        let select = output_settings
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::FlSelection)
            .last()
            .map(|settings| ClientSelectTrace::new(&output_path.join(&settings.output_filename)));
        Self {
            positions,
            rx_counts,
            net_stats,
            tx_data,
            payload_tx,
            model,
            state,
            train,
            select,
        }
    }

    pub fn write_to_file(&mut self) {
        if let Some(writer) = &mut self.positions {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.rx_counts {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.net_stats {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.tx_data {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.payload_tx {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.model {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.state {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.train {
            writer.write_to_file();
        }
        if let Some(writer) = &mut self.select {
            writer.write_to_file();
        }
    }

    pub fn close_files(self) {
        if let Some(writer) = self.positions {
            writer.close_file();
        }
        if let Some(writer) = self.rx_counts {
            writer.close_file();
        }
        if let Some(writer) = self.net_stats {
            writer.close_file();
        }
        if let Some(writer) = self.tx_data {
            writer.close_file();
        }
        if let Some(writer) = self.payload_tx {
            writer.close_file();
        }
        if let Some(writer) = self.model {
            writer.close_file();
        }
        if let Some(writer) = self.state {
            writer.close_file();
        }
        if let Some(writer) = self.train {
            writer.close_file();
        }
        if let Some(writer) = self.select {
            writer.close_file();
        }
    }
}
