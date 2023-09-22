use crate::input::traffic::{DeviceId, TimeStamp};
use crate::input::{dfs, files};
use hashbrown::HashMap;
use std::path::PathBuf;

pub(crate) type Activation = (Vec<TimeStamp>, Vec<TimeStamp>); // (start_time, end_time)

pub(crate) fn read_activation_data(
    activations_file: &PathBuf,
) -> Result<HashMap<DeviceId, Activation>, Box<dyn std::error::Error>> {
    let activation_df = files::read_file(&activations_file)?;
    dfs::extract_activations(&activation_df)
}
