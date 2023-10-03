use crate::input::{dfs, files};
use hashbrown::HashMap;
use pavenet_config::config::base::MapState;
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use std::path::PathBuf;

pub type Trace = (Vec<NodeId>, Vec<MapState>);
pub type TraceMap = HashMap<TimeStamp, Trace>;

pub enum MapReaderType {
    File(MapStateReader),
    Stream(MapStateStreamer),
}

pub trait MapReader {
    fn read_traffic_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>>;
}

pub struct MapStateReader {
    file_path: PathBuf,
}

impl MapStateReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl MapReader for MapStateReader {
    fn read_traffic_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let trace_df = files::read_file(&self.file_path)?;
        dfs::extract_traffic_data(&trace_df)
    }
}

pub struct MapStateStreamer {
    file_path: PathBuf,
    streaming_interval: TimeStamp,
}

impl MapStateStreamer {
    pub fn new(file_path: PathBuf, streaming_interval: TimeStamp) -> Self {
        Self {
            file_path,
            streaming_interval,
        }
    }
}

impl MapReader for MapStateStreamer {
    fn read_traffic_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let start_interval: TimeStamp = step;
        let end_interval: TimeStamp = step + self.streaming_interval;
        let trace_data_df =
            files::stream_parquet_in_interval(&self.file_path, start_interval, end_interval)?;
        dfs::extract_traffic_data(&trace_data_df)
    }
}
