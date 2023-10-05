use super::helper::{convert_list_series_to_vector_timestamps, convert_series_to_node_ids};
use crate::common::columns::{END_TIME, NODE_ID, START_TIME};
use hashbrown::HashMap;
use pavenet_config::config::base::PowerTimes;
use pavenet_config::types::ids::node::NodeId;
use polars::prelude::{col, IntoLazy};
use polars_core::prelude::DataFrame;

pub(crate) fn extract_power_schedule(
    power_df: &DataFrame,
) -> Result<HashMap<NodeId, PowerTimes>, Box<dyn std::error::Error>> {
    let filtered_df = get_filtered_df(power_df)?;
    let device_id_series = filtered_df.column(NODE_ID)?;
    let device_ids_vec: Vec<NodeId> = convert_series_to_node_ids(device_id_series)?;
    let mut power_data_map: HashMap<NodeId, PowerTimes> =
        HashMap::with_capacity(device_ids_vec.len());

    let on_series = filtered_df.column(START_TIME)?;
    let on_time_vec = convert_list_series_to_vector_timestamps(on_series)?;
    let off_series = filtered_df.column(END_TIME)?;
    let off_time_vec = convert_list_series_to_vector_timestamps(off_series)?;

    for (device_id, (on_times, off_times)) in device_ids_vec
        .into_iter()
        .zip(on_time_vec.into_iter().zip(off_time_vec.into_iter()))
    {
        let power_times = (on_times, off_times);
        power_data_map.insert(device_id, power_times);
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
