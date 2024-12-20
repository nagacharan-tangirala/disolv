use std::path::Path;

use arrow::datatypes::Schema;
use serde::Deserialize;

use disolv_core::bucket::TimeMS;

use crate::position::PosWriter;
use crate::rx_counts::RxCountWriter;

pub trait ResultWriter {
    fn schema() -> Schema;
    fn write_to_file(&mut self);
    fn close_file(self);
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxCounts,
    TxData,
    AgentPos,
    NetStat,
    FlState,
    FlModel,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_interval: TimeMS,
    pub output_path: String,
    pub outputs: Vec<Outputs>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Outputs {
    pub output_type: OutputType,
    pub output_filename: String,
}

#[derive(Debug)]
pub struct BasicResults {
    pub positions: PosWriter,
    pub rx_counts: RxCountWriter,
}

impl BasicResults {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = Path::new(&output_settings.output_path);
        let pos_filepath = output_path.join(
            output_settings
                .outputs
                .iter()
                .filter(|output| output.output_type == OutputType::AgentPos)
                .last()
                .cloned()
                .expect("Agent Position settings are missing")
                .output_filename,
        );
        let rx_filepath = output_path.join(
            output_settings
                .outputs
                .iter()
                .filter(|output| output.output_type == OutputType::RxCounts)
                .last()
                .cloned()
                .expect("Rx Counts settings are missing")
                .output_filename,
        );

        Self {
            positions: PosWriter::new(pos_filepath),
            rx_counts: RxCountWriter::new(rx_filepath),
        }
    }

    pub fn close_files(self) {
        self.rx_counts.close_file();
        self.positions.close_file();
    }
}
