use crate::input::{dfs, files};
use hashbrown::HashMap;
use pavenet_config::config::base::{Activation, DeviceId};
use std::path::PathBuf;

pub(crate) fn read_activation_data(
    activations_file: &PathBuf,
) -> Result<HashMap<DeviceId, Activation>, Box<dyn std::error::Error>> {
    let activation_df = files::read_file(&activations_file)?;
    dfs::extract_activations(&activation_df)
}