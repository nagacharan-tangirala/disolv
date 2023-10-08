use crate::common::columns::{NODE_ID, OFF_TIMES, ON_TIMES};
use crate::convert::list_series::to_vec_of_timestamp_vec;
use crate::convert::series::to_nodeid_vec;
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
    let device_ids_vec: Vec<NodeId> = to_nodeid_vec(device_id_series)?;
    let mut power_data_map: HashMap<NodeId, PowerTimes> =
        HashMap::with_capacity(device_ids_vec.len());

    let on_series = filtered_df.column(ON_TIMES)?;
    let on_time_vec = to_vec_of_timestamp_vec(on_series)?;
    let off_series = filtered_df.column(OFF_TIMES)?;
    let off_time_vec = to_vec_of_timestamp_vec(off_series)?;

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
            vec![col(ON_TIMES), col(OFF_TIMES)]
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .collect()?;
    return Ok(filtered_df);
}
