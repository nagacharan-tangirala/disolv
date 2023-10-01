use crate::input::{dfs, files};
use pavenet_config::config::base::TraceMap;
use pavenet_config::config::types::TimeStamp;
use std::path::PathBuf;

pub struct MapStateReader {
    file_path: PathBuf,
    is_stream: bool,
    streaming_interval: TimeStamp,
}

impl MapStateReader {
    pub fn new() -> MapStateReaderBuilder {
        MapStateReaderBuilder::new()
    }

    pub fn read_traffic_data(
        &self,
        step: TimeStamp,
    ) -> Result<TraceMap, Box<dyn std::error::Error>> {
        return if self.is_stream == true {
            self.stream_data(step)
        } else {
            self.read_file()
        };
    }

    fn stream_data(&self, ts: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let start_interval: TimeStamp = ts;
        let end_interval: TimeStamp = ts + self.streaming_interval;
        let trace_data_df =
            files::stream_parquet_in_interval(&self.file_path, start_interval, end_interval)?;
        dfs::extract_traffic_data(&trace_data_df)
    }

    fn read_file(&self) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let trace_df = files::read_file(&self.file_path)?;
        dfs::extract_traffic_data(&trace_df)
    }
}

#[derive(Default)]
pub struct MapStateReaderBuilder {
    file_path: PathBuf,
    is_stream: bool,
    interval: TimeStamp,
}

impl MapStateReaderBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_file(mut self, file_path: PathBuf) -> Self {
        self.file_path = file_path;
        self
    }

    pub fn with_streaming(mut self, is_stream: bool) -> Self {
        self.is_stream = is_stream;
        self
    }

    pub fn interval(mut self, interval: TimeStamp) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> MapStateReader {
        MapStateReader {
            file_path: self.file_path,
            is_stream: self.is_stream,
            streaming_interval: self.interval,
        }
    }
}
