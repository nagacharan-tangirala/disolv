use pavenet_engine::bucket::TimeS;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputType {
    RxData,
    TxData,
    NodePos,
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
    pub output_step: TimeS,
    pub output_path: String,
    pub file_type: FileType,
    pub file_out_config: Vec<FileOutConfig>,
}
