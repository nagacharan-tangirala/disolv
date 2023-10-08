use crate::dfs::power;
use crate::input::files;
use hashbrown::HashMap;
use pavenet_core::types::{NodeId, PowerTimes};
use std::path::PathBuf;

pub(crate) fn read_power_schedule(
    activations_file: &PathBuf,
) -> Result<HashMap<NodeId, PowerTimes>, Box<dyn std::error::Error>> {
    let activation_df = files::read_file(&activations_file)?;
    power::extract_power_schedule(&activation_df)
}
