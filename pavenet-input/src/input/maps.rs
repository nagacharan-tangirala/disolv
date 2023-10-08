use crate::dfs::maps;
use crate::input::files;
use hashbrown::HashMap;
use pavenet_core::structs::MapState;
use pavenet_core::types::{NodeId, TimeStamp};
use std::path::PathBuf;
use typed_builder::TypedBuilder;

pub type TraceMap = HashMap<TimeStamp, HashMap<NodeId, MapState>>;

pub enum MapReaderType {
    File(MapStateReader),
    Stream(MapStateStreamer),
}

pub trait MapFetcher {
    fn fetch_traffic_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>>;
}

#[derive(TypedBuilder)]
pub struct MapStateReader {
    file_path: PathBuf,
}

impl MapFetcher for MapStateReader {
    fn fetch_traffic_data(&self, _step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let trace_df = files::read_file(&self.file_path)?;
        maps::extract_map_states(&trace_df)
    }
}

#[derive(TypedBuilder)]
pub struct MapStateStreamer {
    file_path: PathBuf,
    streaming_interval: TimeStamp,
}

impl MapFetcher for MapStateStreamer {
    fn fetch_traffic_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let start_interval: TimeStamp = step;
        let end_interval: TimeStamp = step + self.streaming_interval;
        let trace_data_df =
            files::stream_parquet_in_interval(&self.file_path, start_interval, end_interval)?;
        maps::extract_map_states(&trace_data_df)
    }
}
