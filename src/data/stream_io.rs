use crate::data::data_io::{DeviceId, TimeStamp};
use crate::data::{df_handler, file_io};
use crate::sim::field::TraceMap;
use crate::sim::vanet::{Link, MultiLinkMap};
use krabmaga::hashbrown::HashMap;
use polars_core::frame::DataFrame;
use std::path::PathBuf;

pub(crate) fn stream_links_in_interval(
    links_file: PathBuf,
    device_id_column: &str,
    neighbour_column: &str,
    interval_begin: u64,
    interval_end: u64,
) -> Result<HashMap<TimeStamp, MultiLinkMap>, Box<dyn std::error::Error>> {
    let links_df: DataFrame =
        file_io::stream_parquet_in_interval(links_file, interval_begin, interval_end)?;
    let streamed_links: HashMap<TimeStamp, MultiLinkMap> =
        df_handler::prepare_dynamic_links(&links_df, device_id_column, neighbour_column)?;
    return Ok(streamed_links);
}

pub(crate) fn stream_positions_in_interval(
    trace_file: PathBuf,
    device_id_column: &str,
    start_interval: u64,
    end_interval: u64,
) -> TraceMap {
    let trace_df =
        match file_io::stream_parquet_in_interval(trace_file, start_interval, end_interval) {
            Ok(trace_df) => trace_df,
            Err(e) => {
                panic!("Error while streaming parquet: {}", e);
            }
        };

    let trace_map: TraceMap = match df_handler::prepare_trace_data(&trace_df, device_id_column) {
        Ok(trace_map) => trace_map,
        Err(e) => {
            panic!("Error while converting trace DF to hashmap: {}", e);
        }
    };
    return trace_map;
}
