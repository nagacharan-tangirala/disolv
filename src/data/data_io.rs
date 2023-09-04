use crate::data::{df_handler, df_utils, file_io};
use crate::utils::constants::{COL_CONTROLLER_ID, COL_COORD_X, COL_COORD_Y};
use krabmaga::hashbrown::HashMap;
use polars_io::SerReader;
use std::path::PathBuf;

pub(crate) type DeviceId = u64;
pub(crate) type TimeStamp = u64;
pub(crate) type Activation = (Vec<TimeStamp>, Vec<TimeStamp>); // (start_time, end_time)
pub(crate) type Trace = (Vec<DeviceId>, Vec<f32>, Vec<f32>, Vec<f32>); // (device_id, x, y, velocity)
pub(crate) type Link = (Vec<DeviceId>, Vec<f32>); // (neighbour_ids, distances)

pub(crate) fn read_activation_data(activations_file: PathBuf) -> HashMap<DeviceId, Activation> {
    let activation_df = match file_io::read_csv_data(activations_file) {
        Ok(activation_df) => activation_df,
        Err(e) => {
            panic!("Error while reading activation data from file: {}", e);
        }
    };

    let activations_map: HashMap<u64, Activation> =
        match df_handler::prepare_device_activations(&activation_df) {
            Ok(activation_map) => activation_map,
            Err(e) => {
                panic!("Error while converting activation DF to hashmap: {}", e);
            }
        };
    return activations_map;
}

pub(crate) fn read_all_positions(
    trace_file: PathBuf,
    device_id_column: &str,
) -> HashMap<TimeStamp, Option<Trace>> {
    let trace_df = match file_io::read_parquet_data(trace_file) {
        Ok(trace_df) => trace_df,
        Err(e) => {
            panic!("Error while reading trace data from the file: {}", e);
        }
    };

    let trace_map: HashMap<u64, Option<Trace>> =
        match df_handler::prepare_trace_data(&trace_df, device_id_column) {
            Ok(trace_map) => trace_map,
            Err(e) => {
                panic!("Error while converting DF to hashmap: {}", e);
            }
        };
    return trace_map;
}

pub(crate) fn read_controller_positions(
    controller_file: PathBuf,
) -> Result<HashMap<DeviceId, (f32, f32)>, Box<dyn std::error::Error>> {
    let controller_df = file_io::read_csv_data(controller_file)?;
    let controller_ids: Vec<DeviceId> =
        df_utils::convert_series_to_integer_vector(&controller_df, COL_CONTROLLER_ID)?;
    let x_positions: Vec<f32> =
        df_utils::convert_series_to_floating_vector(&controller_df, COL_COORD_X)?;
    let y_positions: Vec<f32> =
        df_utils::convert_series_to_floating_vector(&controller_df, COL_COORD_Y)?;

    let mut controller_map: HashMap<DeviceId, (f32, f32)> = HashMap::new();
    for i in 0..controller_ids.len() {
        controller_map.insert(controller_ids[i], (x_positions[i], y_positions[i]));
    }

    return Ok(controller_map);
}

pub(crate) fn read_bs2c_links(b2c_links_file: PathBuf) -> HashMap<DeviceId, DeviceId> {
    let bs2c_links_df = match file_io::read_csv_data(b2c_links_file) {
        Ok(b2c_links_df) => b2c_links_df,
        Err(e) => {
            panic!("Error while reading the bs2c links data from file: {}", e);
        }
    };

    let bs2c_links_map: HashMap<DeviceId, DeviceId> =
        match df_handler::prepare_b2c_links(&bs2c_links_df) {
            Ok(b2c_links_map) => b2c_links_map,
            Err(e) => {
                panic!("Error while converting BS2C links DF to hashmap: {}", e);
            }
        };
    return bs2c_links_map;
}

pub(crate) fn read_all_links(
    links_file: PathBuf,
    device_id_column: &str,
    neighbour_column: &str,
) -> Result<HashMap<TimeStamp, HashMap<DeviceId, Link>>, Box<dyn std::error::Error>> {
    let links_df = file_io::read_csv_data(links_file)?;
    let static_links: HashMap<u64, HashMap<u64, Link>> =
        df_handler::prepare_static_links(&links_df, device_id_column, neighbour_column)?;
    return Ok(static_links);
}
