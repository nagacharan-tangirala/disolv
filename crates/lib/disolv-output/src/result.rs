use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use arrow::csv::Writer;
use arrow::datatypes::{Schema, SchemaRef};
use arrow::record_batch::RecordBatchWriter;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use serde::Deserialize;

use disolv_core::bucket::TimeMS;

use crate::tables::model::ModelTrace;
use crate::tables::net::NetStatWriter;
use crate::tables::payload::PayloadTraceWriter;
use crate::tables::position::PositionWriter;
use crate::tables::rx::RxCountWriter;
use crate::tables::state::StateTrace;
use crate::tables::tx::TxDataWriter;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxCounts,
    TxStats,
    AgentPos,
    NetStat,
    FlState,
    FlModel,
    PayloadTx,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_interval: TimeMS,
    pub output_path: String,
    pub outputs: Vec<Outputs>,
    pub scenario_id: u32,
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
pub enum WriterType {
    Parquet(WriterParquet),
    Csv(WriterCsv),
}

impl WriterType {
    pub fn new(file_name: &PathBuf, schema: Schema) -> Self {
        if file_name.exists() {
            match std::fs::remove_file(file_name) {
                Ok(_) => {}
                Err(e) => panic!("Error deleting file: {}", e),
            }
        }
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some("parquet") => WriterType::Parquet(WriterParquet::new(file_name, schema)),
                Some("csv") => WriterType::Csv(WriterCsv::new(file_name)),
                _ => panic!("Invalid file extension"),
            },
            None => panic!("Invalid file extension"),
        }
    }
}

#[derive(Debug)]
pub struct WriterParquet {
    pub writer: ArrowWriter<File>,
}

impl WriterParquet {
    fn new(file_name: &PathBuf, schema: Schema) -> Self {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let output_file = match File::create(file_name) {
            Ok(file) => file,
            Err(_) => panic!("Failed to create links file to write"),
        };
        let writer = match ArrowWriter::try_new(output_file, SchemaRef::from(schema), Some(props)) {
            Ok(writer) => writer,
            Err(_) => panic!("Failed to create links file writer"),
        };
        Self { writer }
    }

    pub fn close(self) {
        self.writer.close().expect("Failed to close parquet file");
    }
}

#[derive(Debug)]
pub struct WriterCsv {
    pub writer: Writer<File>,
}

impl WriterCsv {
    fn new(file_name: &PathBuf) -> Self {
        let writer = Writer::new(File::create(file_name).expect("failed to create file"));
        Self { writer }
    }

    pub fn close(self) {
        self.writer.close().expect("failed to close csv file");
    }
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
}

impl Results {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = Path::new(&output_settings.output_path)
            .join(output_settings.scenario_id.to_string())
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
            .filter(|output| output.output_type == OutputType::PayloadTx)
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
        Self {
            positions,
            rx_counts,
            net_stats,
            tx_data,
            payload_tx,
            model,
            state,
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
    }
}
