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
) -> Result<HashMap<u64, (f32, f32)>, Box<dyn std::error::Error>> {
    let controller_df = file_io::read_csv_data(controller_file)?;
    let controller_ids: Vec<u64> =
        df_handler::convert_series_to_integer_vector(&controller_df, COL_CONTROLLER_ID)?;
    let x_positions: Vec<f32> =
        df_handler::convert_series_to_floating_vector(&controller_df, COL_COORD_X)?;
    let y_positions: Vec<f32> =
        df_handler::convert_series_to_floating_vector(&controller_df, COL_COORD_Y)?;

    let mut controller_map: HashMap<u64, (f32, f32)> = HashMap::new();
    for i in 0..controller_ids.len() {
        controller_map.insert(controller_ids[i], (x_positions[i], y_positions[i]));
    }
    return Ok(controller_map);
}

// pub(crate) fn read_dynamic_links(
//     &self,
//     links_file: PathBuf,
//     links_column: &str,
// ) -> Result<HashMap<i64, i64>, Box<dyn std::error::Error>> {
//     let mut links_reader: ParquetDataReader = ParquetDataReader::new(links_file);
//     let links_df = match links_reader.read_data() {
//         Ok(controller_df) => controller_df,
//         Err(e) => {
//             panic!("Error while reading the controller data from file: {}", e);
//         }
//     };
// }

pub(crate) fn read_static_links() {}
