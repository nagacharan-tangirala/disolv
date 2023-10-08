use pavenet_core::types::{NodeId, RoadId, TimeStamp, Velocity};
use polars_core::prelude::Series;

pub(crate) fn to_i64_vec(series: &Series) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<i64> = series_to_option_vec.iter().filter_map(|x| *x).collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn to_f32_vec(series: &Series) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
    let option_vec_to_vec: Vec<f32> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| x as f32)
        .collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn to_nodeid_vec(series: &Series) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<NodeId> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| NodeId::from(x))
        .collect::<Vec<NodeId>>();
    return Ok(option_vec_to_vec);
}

pub(crate) fn to_timestamp_vec(
    series: &Series,
) -> Result<Vec<TimeStamp>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<TimeStamp> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| TimeStamp::from(x))
        .collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn to_roadid_vec(series: &Series) -> Result<Vec<RoadId>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<RoadId> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| RoadId::from(x))
        .collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn to_velocity_vec(
    series: &Series,
) -> Result<Vec<Velocity>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
    let option_vec_to_vec: Vec<Velocity> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| Velocity::from(x))
        .collect();
    return Ok(option_vec_to_vec);
}
