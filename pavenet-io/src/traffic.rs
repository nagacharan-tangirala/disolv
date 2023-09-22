use crate::{dfs, files};
use hashbrown::HashMap;
use std::path::PathBuf;

pub(crate) type DeviceId = u64;
pub(crate) type TimeStamp = u64;
pub(crate) type Trace = (Vec<DeviceId>, Vec<f32>, Vec<f32>, Vec<f32>); // (device_id, x, y, velocity)
pub(crate) type TraceMap = HashMap<TimeStamp, Option<Trace>>;

pub struct TrafficReader {
    trace_file: PathBuf,
    device_id_column: String,
    is_stream: bool,
    streaming_interval: TimeStamp,
}

impl TrafficReader {
    pub(crate) fn builder(trace_file: &PathBuf) -> TrafficReaderBuilder {
        TrafficReaderBuilder::new(trace_file.clone())
    }

    pub(crate) fn read_traffic_data(
        &self,
        step: TimeStamp,
    ) -> Result<TraceMap, Box<dyn std::error::Error>> {
        return if self.is_stream == true {
            self.stream_data(step)
        } else {
            self.read_file()
        };
    }

    fn stream_data(&self, step: TimeStamp) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let start_interval: TimeStamp = step;
        let end_interval: TimeStamp = step + self.streaming_interval;
        let trace_data_df =
            files::stream_parquet_in_interval(&self.trace_file, start_interval, end_interval)?;
        dfs::extract_traffic_data(&trace_data_df, &self.device_id_column)
    }

    fn read_file(&self) -> Result<TraceMap, Box<dyn std::error::Error>> {
        let trace_df = files::read_file(&self.trace_file)?;
        dfs::extract_traffic_data(&trace_df, &self.device_id_column)
    }
}

#[derive(Default)]
pub struct TrafficReaderBuilder {
    trace_file: PathBuf,
    device_id_column: String,
    is_stream: bool,
    interval: TimeStamp,
}

impl TrafficReaderBuilder {
    pub fn new(trace_file: PathBuf) -> Self {
        Self::default().trace_file(trace_file)
    }

    pub fn trace_file(mut self, trace_file: PathBuf) -> Self {
        self.trace_file = trace_file;
        self
    }

    pub fn device_id_column(mut self, device_id_column: String) -> Self {
        self.device_id_column = device_id_column;
        self
    }

    pub fn stream(mut self, is_stream: bool) -> Self {
        self.is_stream = is_stream;
        self
    }

    pub fn interval(mut self, interval: TimeStamp) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> TrafficReader {
        TrafficReader {
            trace_file: self.trace_file,
            device_id_column: self.device_id_column,
            is_stream: self.is_stream,
            streaming_interval: self.interval,
        }
    }
}
