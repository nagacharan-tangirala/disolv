pub mod data {
    use crate::file_reader::read_file;
    use crate::power::df::extract_power_schedule;
    use pavenet_engine::bucket::TimeMS;
    use pavenet_engine::hashbrown::HashMap;
    use pavenet_engine::node::NodeId;
    use std::path::PathBuf;

    pub type PowerTimes = (Vec<TimeMS>, Vec<TimeMS>);
    pub fn read_power_schedule(
        power_schedule_file: &PathBuf,
    ) -> Result<HashMap<NodeId, PowerTimes>, Box<dyn std::error::Error>> {
        let activation_df = read_file(power_schedule_file)?;
        extract_power_schedule(&activation_df)
    }
}

pub(super) mod df {
    use crate::columns::{NODE_ID, OFF_TIMES, ON_TIMES};
    use crate::converter::series::{to_nodeid_vec, to_timestamp_vec};
    use crate::power::data::PowerTimes;

    use pavenet_engine::bucket::TimeMS;
    use pavenet_engine::hashbrown::HashMap;
    use pavenet_engine::node::NodeId;
    use polars::prelude::DataFrame;

    pub(crate) fn extract_power_schedule(
        power_df: &DataFrame,
    ) -> Result<HashMap<NodeId, PowerTimes>, Box<dyn std::error::Error>> {
        let device_id_series = power_df.column(NODE_ID)?;
        let device_ids_vec: Vec<NodeId> = to_nodeid_vec(device_id_series)?;

        let on_series = power_df.column(ON_TIMES)?;
        let on_time_vec: Vec<TimeMS> = to_timestamp_vec(on_series)?;
        let off_series = power_df.column(OFF_TIMES)?;
        let off_time_vec: Vec<TimeMS> = to_timestamp_vec(off_series)?;

        let mut power_data_map: HashMap<NodeId, PowerTimes> =
            HashMap::with_capacity(device_ids_vec.len());

        for (idx, device_id) in device_ids_vec.into_iter().enumerate() {
            power_data_map
                .entry(device_id)
                .or_insert((Vec::new(), Vec::new()))
                .0
                .push(on_time_vec[idx]);
            power_data_map
                .entry(device_id)
                .or_insert((Vec::new(), Vec::new()))
                .1
                .push(off_time_vec[idx]);
        }
        Ok(power_data_map)
    }
}

#[cfg(test)]
mod tests {}
