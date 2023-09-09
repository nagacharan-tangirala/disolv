use crate::reader::{df_handler, files};
use krabmaga::hashbrown::HashMap;
use std::path::PathBuf;

pub(crate) type DeviceId = u64;
pub(crate) type TimeStamp = u64;
pub(crate) type Activation = (Vec<TimeStamp>, Vec<TimeStamp>); // (start_time, end_time)

pub(crate) fn read_activation_data(activations_file: PathBuf) -> HashMap<DeviceId, Activation> {
    let activation_df = match files::read_csv_data(activations_file) {
        Ok(activation_df) => activation_df,
        Err(e) => {
            panic!("Error while reading activation data from file: {}", e);
        }
    };

    let activations_map: HashMap<DeviceId, Activation> =
        match df_handler::prepare_device_activations(&activation_df) {
            Ok(activation_map) => activation_map,
            Err(e) => {
                panic!("Error while converting activation DF to hashmap: {}", e);
            }
        };
    return activations_map;
}
