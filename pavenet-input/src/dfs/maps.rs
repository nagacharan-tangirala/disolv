use crate::common::columns::*;
use crate::convert::series::{
    to_f32_vec, to_nodeid_vec, to_roadid_vec, to_timestamp_vec, to_velocity_vec,
};
use crate::input::maps::TraceMap;
use hashbrown::HashMap;
use pavenet_core::structs::{MapState, Point2D};
use pavenet_core::types::{NodeId, RoadId, TimeStamp, Velocity};
use polars::prelude::{col, lit, IntoLazy};
use polars_core::error::ErrString;
use polars_core::frame::DataFrame;
use polars_core::prelude::*;

mod mandatory {
    use crate::common::columns::*;

    pub const COLUMNS: [&str; 4] = [TIME_STEP, NODE_ID, COORD_X, COORD_Y];
}

mod optional {
    use crate::common::columns::*;

    pub const COLUMNS: [&str; 3] = [VELOCITY, COORD_Z, ROAD_ID];
}

pub(crate) fn extract_map_states(
    trace_df: &DataFrame,
) -> Result<TraceMap, Box<dyn std::error::Error>> {
    validate_trace_df(trace_df)?;
    let filtered_df = group_by_time(&trace_df)?;

    let ts_series = filtered_df.column(TIME_STEP)?;
    let time_stamps: Vec<TimeStamp> = to_timestamp_vec(ts_series)?;

    let mut trace_map: TraceMap = HashMap::with_capacity(time_stamps.len());
    for time_stamp in time_stamps.iter() {
        let ts_df = filtered_df
            .clone()
            .lazy()
            .filter(col(TIME_STEP).eq(lit(time_stamp.as_u64())))
            .collect()?;

        if ts_df.height() == 0 {
            trace_map.insert(*time_stamp, HashMap::new());
            continue;
        }

        let node_id_series = ts_df.column(NODE_ID)?;
        let node_ids: Vec<NodeId> = to_nodeid_vec(node_id_series)?;

        let mut map_states: Vec<MapState> = extract_mandatory_data(&ts_df)?;
        add_optional_data(&ts_df, &mut map_states)?;

        let mut trace: HashMap<NodeId, MapState> = HashMap::with_capacity(node_ids.len());
        for (idx, node_id) in node_ids.iter().enumerate() {
            trace.insert(*node_id, map_states[idx]);
        }
        trace_map.entry(*time_stamp).or_insert(trace);
    }
    return Ok(trace_map);
}

fn validate_trace_df(df: &DataFrame) -> Result<(), PolarsError> {
    for column in mandatory::COLUMNS.iter() {
        if !df.get_column_names().contains(column) {
            return Err(PolarsError::ColumnNotFound(ErrString::from(
                column.to_string(),
            )));
        }
    }
    return Ok(());
}

fn group_by_time(df: &DataFrame) -> PolarsResult<DataFrame> {
    let agg_columns = columns_to_aggregate(df);
    return df
        .clone()
        .lazy()
        .group_by([col(TIME_STEP)])
        .agg(agg_columns.into_iter().collect::<Vec<_>>())
        .collect();
}

fn columns_to_aggregate(df: &DataFrame) -> Vec<polars::prelude::Expr> {
    let mut columns_in_df = df.get_column_names();
    columns_in_df.remove(columns_in_df.iter().position(|x| *x == TIME_STEP).unwrap());
    return columns_in_df
        .into_iter()
        .map(|x| col(x))
        .collect::<Vec<_>>();
}

fn extract_mandatory_data(df: &DataFrame) -> Result<Vec<MapState>, Box<dyn std::error::Error>> {
    let x_series = df.column(COORD_X)?;
    let x_positions: Vec<f32> = to_f32_vec(x_series)?;
    let y_series = df.column(COORD_Y)?;
    let y_positions: Vec<f32> = to_f32_vec(y_series)?;

    let map_states: Vec<MapState> = x_positions
        .iter()
        .zip(y_positions.iter())
        .map(|(x, y)| MapState::builder().pos(Point2D { x: *x, y: *y }).build())
        .collect();
    return Ok(map_states);
}

fn add_optional_data(
    df: &DataFrame,
    map_states: &mut Vec<MapState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let optional_columns = get_optional_columns(df);
    for optional_col in optional_columns.into_iter() {
        match optional_col {
            COORD_Z => {
                let z_series = df.column(COORD_Z)?;
                let z_positions: Vec<f32> = to_f32_vec(z_series)?;
                map_states
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, map_state)| {
                        map_state.z = Some(z_positions[idx]);
                    });
            }
            VELOCITY => {
                let vel_series = df.column(VELOCITY)?;
                let velocities: Vec<Velocity> = to_velocity_vec(vel_series)?;
                map_states
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, map_state)| {
                        map_state.velocity = Some(velocities[idx]);
                    });
            }
            ROAD_ID => {
                let road_id_series = df.column(ROAD_ID)?;
                let road_ids: Vec<RoadId> = to_roadid_vec(road_id_series)?;
                map_states
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, map_state)| {
                        map_state.road_id = Some(road_ids[idx]);
                    });
            }
            _ => return Err("Invalid column name".into()),
        }
    }
    return Ok(());
}

fn get_optional_columns(df: &DataFrame) -> Vec<&str> {
    return df
        .get_column_names()
        .into_iter()
        .filter(|col| optional::COLUMNS.contains(&col))
        .map(|col| col)
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::files;
    use std::path::PathBuf;
    use std::time;

    fn get_test_df() -> DataFrame {
        let test_file_path =
            PathBuf::from("/home/charan/rustrover/pavenet-input/test/data/test_traces.parquet");
        return files::read_file(&test_file_path).unwrap();
    }

    fn benchmark_trace_df() {
        let file_name =
            "/home/charan/rustrover/pavenet/pavenet-input/test/data/test_traces.parquet";
        let interval_begin = TimeStamp::from(0u64);
        let interval_end = TimeStamp::from(7200u64);
        let time_s = time::Instant::now();
        let df = files::stream_parquet_in_interval(
            &PathBuf::from(file_name),
            interval_begin,
            interval_end,
        )
        .unwrap();
        let df_height = df.clone().height();
        let map_data = extract_map_states(&df).unwrap();
        let elapsed = time_s.elapsed();
        println!(
            "DF rows: {:?}, Map Keys: {:?}, Time: {:?}",
            df_height,
            map_data.len(),
            elapsed
        );
    }

    #[test]
    fn test_extract_traffic_data() {
        let geo_df = get_test_df();
        let trace_map = extract_map_states(&geo_df).unwrap();
        // assert_eq!(trace_map.len(), 2);
        // assert_eq!(trace_map.get(&TimeStamp::from(0)).unwrap().len(), 2);
        // assert_eq!(trace_map.get(&TimeStamp::from(1)).unwrap().len(), 2);
    }
}
