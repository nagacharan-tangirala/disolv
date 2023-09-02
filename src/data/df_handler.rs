use crate::data::data_io::{Activation, Trace};
use crate::utils::constants::{
    COORD_X, COORD_Y, DEVICE_ID, END_TIME, START_TIME, TIME_STEP, VEHICLE_ID, VELOCITY,
};
use krabmaga::hashbrown::HashMap;
use log::info;
use polars::prelude::{col, lit, IntoLazy, PolarsResult};
use polars_core::frame::DataFrame;
use polars_core::prelude::Series;

pub(crate) fn prepare_trace_data(
    trace_df: &DataFrame,
    device_id_column: &str,
) -> Result<HashMap<u64, Trace>, Box<dyn std::error::Error>> {
    let filtered_df: DataFrame = trace_df
        .clone() // Clones of DataFrames are cheap. Don't bother optimizing this.
        .lazy()
        .groupby([col(TIME_STEP)])
        .agg(
            vec![
                col(device_id_column),
                col(COORD_X),
                col(COORD_Y),
                col(VELOCITY),
            ]
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .collect()?;

    let time_stamps: Vec<u64> = convert_series_to_integer_vector(&filtered_df, TIME_STEP)?;
    let mut time_stamp_traces: HashMap<u64, Trace> = HashMap::new();
    for time_stamp in time_stamps.iter() {
        let ts_df = filtered_df
            .clone()
            .lazy()
            .filter(col(TIME_STEP).eq(lit(*time_stamp)))
            .collect()
            .unwrap();

        if ts_df.height() == 0 {
            time_stamp_traces.insert(*time_stamp, (vec![], vec![], vec![], vec![]));
            continue;
        }
        let time_steps: Vec<u64> = convert_series_to_integer_vector(&ts_df, device_id_column)?;
        let x_positions: Vec<f32> = convert_series_to_floating_vector(&ts_df, COORD_X)?;
        let y_positions: Vec<f32> = convert_series_to_floating_vector(&ts_df, COORD_Y)?;
        let velocities: Vec<f32> = convert_series_to_floating_vector(&ts_df, VELOCITY)?;

        let trace: Trace = (time_steps, x_positions, y_positions, velocities);
        time_stamp_traces.insert(*time_stamp, trace);
    }
    return Ok(time_stamp_traces);
}

pub(crate) fn prepare_device_activations(
    activation_df: &DataFrame,
) -> Result<HashMap<u64, Activation>, Box<dyn std::error::Error>> {
    let filtered_df = activation_df
        .clone() // Clones of DataFrames are cheap. Don't bother optimizing this.
        .lazy()
        .groupby([col(DEVICE_ID)])
        .agg(
            vec![col(START_TIME), col(END_TIME)]
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .collect()
        .unwrap();

    let device_ids_vec: Vec<u64> = convert_series_to_integer_vector(&filtered_df, DEVICE_ID)?;

    let mut activation_dfs: HashMap<u64, Activation> = HashMap::new();
    for device_id in device_ids_vec.iter() {
        let device_df = filtered_df
            .clone()
            .lazy()
            .filter(col(DEVICE_ID).eq(lit(*device_id)))
            .collect()
            .unwrap();

        let activation_timings = match convert_series_to_integer_vector(&device_df, START_TIME) {
            Ok(timings) => timings,
            Err(e) => return Err(e.into()),
        };
        let deactivation_timings = match convert_series_to_integer_vector(&device_df, END_TIME) {
            Ok(timings) => timings,
            Err(e) => return Err(e.into()),
        };
        let activation_data: Activation = (activation_timings, deactivation_timings);

        activation_dfs.insert(*device_id, activation_data);
    }
    return Ok(activation_dfs);
}

pub(crate) fn convert_series_to_integer_vector(
    df: &DataFrame,
    column_name: &str,
) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let column_as_series: &Series = match df.columns([column_name])?.get(0) {
        Some(series) => *series,
        None => return Err("Error in the column name".into()),
    };
    let series_to_list: Series = column_as_series.explode()?;
    let list_to_option_vec: Vec<Option<i64>> = series_to_list.i64()?.to_vec();
    let option_vec_to_vec: Vec<u64> = list_to_option_vec
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
    let series_to_list: PolarsResult<Series> = column_as_series.explode();
    let list_to_option_vec: Vec<Option<f64>> = series_to_list.unwrap().f64()?.to_vec();
    let option_vec_to_vec: Vec<f32> = list_to_option_vec
        .iter()
        .map(|x| x.unwrap() as f32) // todo! lossy casting but fine for the value range we have.
        .collect::<Vec<f32>>();

    if option_vec_to_vec.iter().map(|x| x < &0.).any(|x| x) {
        return Err("Error in converting series to vector".into());
    }
    return Ok(option_vec_to_vec);
}
