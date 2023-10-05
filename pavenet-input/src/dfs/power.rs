use super::helper::{convert_series_to_node_ids, convert_series_to_timestamps};
use crate::common::columns::{END_TIME, NODE_ID, START_TIME};
use hashbrown::HashMap;
use pavenet_config::config::base::PowerTimes;
use pavenet_config::types::ids::node::NodeId;
use polars::prelude::{col, lit, IntoLazy};
use polars_core::prelude::DataFrame;

pub(crate) fn extract_power_schedule(
    power_df: &DataFrame,
) -> Result<HashMap<NodeId, PowerTimes>, Box<dyn std::error::Error>> {
    let filtered_df = get_filtered_df(power_df)?;
    let device_ids_vec: Vec<NodeId> = convert_series_to_node_ids(&filtered_df, NODE_ID)?;
    let mut power_data_map: HashMap<NodeId, PowerTimes> =
        HashMap::with_capacity(device_ids_vec.len());

    for device_id in device_ids_vec.iter() {
        let device_df = filtered_df
            .clone()
            .lazy()
            .filter(col(NODE_ID).eq(lit(device_id.as_u32())))
            .collect()
            .unwrap();
        let on_times = convert_series_to_timestamps(&device_df, START_TIME)?;
        let off_times = convert_series_to_timestamps(&device_df, END_TIME)?;
        let power_times: PowerTimes = (on_times, off_times);
        power_data_map.insert(*device_id, power_times);
    }
    return Ok(power_data_map);
}

fn get_filtered_df(power_df: &DataFrame) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let filtered_df = power_df
        .clone()
        .lazy()
        .group_by([col(NODE_ID)])
        .agg(
            vec![col(START_TIME), col(END_TIME)]
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .collect()?;
    return Ok(filtered_df);
}
