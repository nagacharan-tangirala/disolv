use crate::common::columns::*;
use hashbrown::HashMap;
use pavenet_config::config::base::{
    Activation, DeviceId, LinkMap, MultiLinkMap, TimeStamp, Trace, TraceMap,
};
use polars::prelude::{col, lit, IntoLazy};
use polars_core::frame::DataFrame;
use polars_core::prelude::*;

pub(crate) fn extract_traffic_data(
    geo_df: &DataFrame,
    device_id_column: &str,
) -> Result<TraceMap, Box<dyn std::error::Error>> {
    let filtered_df: DataFrame = geo_df
        .clone() // Clones of DataFrames are cheap. Don't bother optimizing this.
        .lazy()
        .group_by([col(COL_TIME_STEP)])
        .agg(
            vec![
                col(device_id_column),
                col(COL_COORD_X),
                col(COL_COORD_Y),
                col(COL_VELOCITY),
            ]
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .collect()?;

    let time_stamps: Vec<TimeStamp> =
        convert_series_to_integer_vector(&filtered_df, COL_TIME_STEP)?;

    let mut time_stamp_traces: TraceMap = HashMap::with_capacity(time_stamps.len());
    for time_stamp in time_stamps.iter() {
        let ts_df = filtered_df
            .clone()
            .lazy()
            .filter(col(COL_TIME_STEP).eq(lit(*time_stamp)))
            .collect()?;

        if ts_df.height() == 0 {
            time_stamp_traces.entry(*time_stamp).or_insert(None);
            continue;
        }
        let device_ids: Vec<DeviceId> = convert_series_to_integer_vector(&ts_df, device_id_column)?;
        let x_positions: Vec<f32> = convert_series_to_floating_vector(&ts_df, COL_COORD_X)?;
        let y_positions: Vec<f32> = convert_series_to_floating_vector(&ts_df, COL_COORD_Y)?;
        let velocities: Vec<f32> = convert_series_to_floating_vector(&ts_df, COL_VELOCITY)?;

        let trace: Trace = (device_ids, x_positions, y_positions, velocities);
        time_stamp_traces.entry(*time_stamp).or_insert(Some(trace));
    }
    return Ok(time_stamp_traces);
}

pub(crate) fn extract_activations(
    activation_df: &DataFrame,
) -> Result<HashMap<DeviceId, Activation>, Box<dyn std::error::Error>> {
    let filtered_df = activation_df
        .clone() // Clones of DataFrames are cheap. Don't bother optimizing this.
        .lazy()
        .group_by([col(COL_DEVICE_ID)])
        .agg(
            vec![col(COL_START_TIME), col(COL_END_TIME)]
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .collect()?;

    let device_ids_vec: Vec<u64> = convert_series_to_integer_vector(&filtered_df, COL_DEVICE_ID)?;

    let mut activation_data_map: HashMap<DeviceId, Activation> =
        HashMap::with_capacity(device_ids_vec.len());

    for device_id in device_ids_vec.iter() {
        let device_df = filtered_df
            .clone()
            .lazy()
            .filter(col(COL_DEVICE_ID).eq(lit(*device_id)))
            .collect()
            .unwrap();

        let activation_timings = match convert_series_to_integer_vector(&device_df, COL_START_TIME)
        {
            Ok(timings) => timings,
            Err(e) => return Err(e.into()),
        };
        let deactivation_timings = match convert_series_to_integer_vector(&device_df, COL_END_TIME)
        {
            Ok(timings) => timings,
            Err(e) => return Err(e.into()),
        };
        let activation_data: Activation = (activation_timings, deactivation_timings);
        activation_data_map.insert(*device_id, activation_data);
    }
    return Ok(activation_data_map);
}

pub(crate) fn extract_single_links(
    links_df: &DataFrame,
    device_id_column: &str,
    neighbour_column: &str,
) -> Result<LinkMap, Box<dyn std::error::Error>> {
    let base_stations: Vec<DeviceId> =
        convert_series_to_integer_vector(&links_df, device_id_column)?;
    let controller_ids: Vec<DeviceId> =
        convert_series_to_integer_vector(&links_df, neighbour_column)?;

    let mut link_map: LinkMap = HashMap::new();
    link_map.reserve(base_stations.len());
    for i in 0..base_stations.len() {
        link_map.insert(base_stations[i], controller_ids[i]);
    }
    return Ok(link_map);
}

pub(crate) fn extract_multiple_links(
    links_df: &DataFrame,
    device_id_column: &str,
    neighbour_column: &str,
) -> Result<MultiLinkMap, Box<dyn std::error::Error>> {
    let device_ids: Vec<DeviceId> = convert_series_to_integer_vector(&links_df, device_id_column)?;
    let mut multi_link_map: MultiLinkMap = HashMap::with_capacity(device_ids.len());
    for device_id in device_ids.iter() {
        let device_df = links_df
            .clone()
            .lazy()
            .filter(col(device_id_column).eq(lit(*device_id)))
            .collect()?;

        let neighbour_string: String = match device_df.columns([neighbour_column])?.get(0) {
            Some(series) => series.get(0)?.to_string(),
            None => return Err("Error in reading neighbour column".into()),
        };

        let distance_string: String = match device_df.columns([COL_DISTANCES])?.get(0) {
            Some(series) => series.get(0)?.to_string(),
            None => return Err("Error in reading distance column".into()),
        };

        let neighbour_ids: Vec<u64> = convert_string_to_integer_vector(neighbour_string.as_str())?;
        let distances: Vec<f32> = convert_string_to_floating_vector(distance_string.as_str())?;

        multi_link_map.insert(*device_id, (neighbour_ids, distances));
    }
    return Ok(multi_link_map);
}

pub(crate) fn extract_link_traces(
    links_df: &DataFrame,
    device_id_column: &str,
    neighbour_column: &str,
) -> Result<HashMap<TimeStamp, MultiLinkMap>, Box<dyn std::error::Error>> {
    let filtered_df: DataFrame = links_df
        .clone() // Clones of DataFrames are cheap. Don't bother optimizing this.
        .lazy()
        .group_by([col(COL_TIME_STEP)])
        .agg(
            vec![
                col(device_id_column),
                col(neighbour_column),
                col(COL_DISTANCES),
            ]
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .collect()?;

    let time_stamps: Vec<TimeStamp> =
        convert_series_to_integer_vector(&filtered_df, COL_TIME_STEP)?;

    let mut dynamic_links: HashMap<TimeStamp, MultiLinkMap> =
        HashMap::with_capacity(time_stamps.len());

    for time_stamp in time_stamps.iter() {
        let ts_df = links_df
            .clone()
            .lazy()
            .filter(col(COL_TIME_STEP).eq(lit(*time_stamp)))
            .collect()?;

        let device_ids: Vec<DeviceId> = convert_series_to_integer_vector(&ts_df, device_id_column)?;
        let mut device_map: MultiLinkMap = HashMap::with_capacity(device_ids.len());

        let neighbour_list: &ListChunked = match ts_df.columns([neighbour_column])?.get(0) {
            Some(series) => series.list()?,
            None => return Err("Error in reading neighbour column".into()),
        };
        let distance_list: &ListChunked = match ts_df.columns([COL_DISTANCES])?.get(0) {
            Some(series) => series.list()?,
            None => return Err("Error in reading distance column".into()),
        };

        for (idx, (neighbour, distance)) in
            neighbour_list.into_iter().zip(distance_list).enumerate()
        {
            let neighbour_opt_vec: Vec<Option<i64>> = neighbour.unwrap().i64()?.to_vec();
            let neighbour_vec: Vec<DeviceId> = neighbour_opt_vec
                .iter()
                .map(|x| x.unwrap() as u64)
                .collect();
            let distance_opt_vec: Vec<Option<f64>> = distance.unwrap().f64()?.to_vec();
            let distance_vec: Vec<f32> =
                distance_opt_vec.iter().map(|x| x.unwrap() as f32).collect();
            device_map.insert(device_ids[idx], (neighbour_vec, distance_vec));
        }
        dynamic_links.insert(*time_stamp, device_map);
    }
    return Ok(dynamic_links);
}

pub(crate) fn convert_series_to_integer_vector(
    df: &DataFrame,
    column_name: &str,
) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let column_as_series: &Series = match df.columns([column_name])?.get(0) {
        Some(series) => *series,
        None => return Err("Error in the column name".into()),
    };
    let list_to_series: Series = column_as_series.explode()?;
    let series_to_option_vec: Vec<Option<i64>> = list_to_series.i64()?.to_vec();
    let option_vec_to_vec: Vec<u64> = series_to_option_vec
        .iter()
        .map(|x| x.unwrap() as u64) // todo! unsafe casting but fine for the value range we have.
        .collect::<Vec<u64>>();
    return Ok(option_vec_to_vec);
}

pub(crate) fn convert_series_to_floating_vector(
    df: &DataFrame,
    column_name: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let column_as_series: &Series = match df.columns([column_name])?.get(0) {
        Some(series) => *series,
        None => return Err("Error in the column name".into()),
    };
    let list_to_series: Series = column_as_series.explode()?;
    let series_to_option_vec: Vec<Option<f64>> = list_to_series.f64()?.to_vec();
    let option_vec_to_vec: Vec<f32> = series_to_option_vec
        .iter()
        .map(|x| x.unwrap() as f32) // todo! lossy casting but fine for the value range we have.
        .collect::<Vec<f32>>();
    return Ok(option_vec_to_vec);
}

pub(crate) fn convert_string_to_integer_vector(
    input_str: &str,
) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let input_str = input_str.replace("\"", "");
    let mut output_vec: Vec<u64> = Vec::new();
    let split_str: Vec<&str> = input_str.split(" ").collect();
    for s in split_str.iter() {
        output_vec.push(s.parse::<u64>()?);
    }
    return Ok(output_vec);
}

pub(crate) fn convert_string_to_floating_vector(
    input_str: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let input_str = input_str.replace("\"", "");
    let mut output_vec: Vec<f32> = Vec::new();
    let split_str: Vec<&str> = input_str.split(" ").collect();
    for s in split_str.iter() {
        output_vec.push(s.parse::<f32>()?);
    }
    return Ok(output_vec);
}
