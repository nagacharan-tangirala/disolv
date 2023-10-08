use pavenet_core::types::NodeId;
use pavenet_core::types::TimeStamp;
use polars_core::prelude::Series;
use std::slice;

pub(crate) fn to_vec_of_nodeid_vec(
    list_series: &Series,
) -> Result<Vec<Vec<NodeId>>, Box<dyn std::error::Error>> {
    let mut result_vec: Vec<Vec<NodeId>> = Vec::with_capacity(list_series.len());
    for n in list_series.iter() {
        let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
        let x_vec = x.explode()?.i64()?.to_vec();
        let x_vec: Vec<NodeId> = x_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| NodeId::from(x))
            .collect();
        result_vec.push(x_vec);
    }
    Ok(result_vec)
}

pub(crate) fn to_vec_of_f32_vec(
    list_series: &Series,
) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let mut result_vec: Vec<Vec<f32>> = Vec::with_capacity(list_series.len());
    for n in list_series.iter() {
        let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
        let x_vec = x.explode()?.f64()?.to_vec();
        let x_vec: Vec<f32> = x_vec.iter().filter_map(|x| *x).map(|x| x as f32).collect();
        result_vec.push(x_vec);
    }
    Ok(result_vec)
}

pub(crate) fn to_vec_of_i64_vec(
    list_series: &Series,
) -> Result<Vec<Vec<i64>>, Box<dyn std::error::Error>> {
    let mut result_vec: Vec<Vec<i64>> = Vec::with_capacity(list_series.len());
    for n in list_series.iter() {
        let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
        let x_vec = x.explode()?.i64()?.to_vec();
        let x_vec: Vec<i64> = x_vec.iter().filter_map(|x| *x).collect();
        result_vec.push(x_vec);
    }
    Ok(result_vec)
}

pub(crate) fn to_vec_of_timestamp_vec(
    list_series: &Series,
) -> Result<Vec<Vec<TimeStamp>>, Box<dyn std::error::Error>> {
    let mut result_vec: Vec<Vec<TimeStamp>> = Vec::with_capacity(list_series.len());
    for n in list_series.iter() {
        let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
        let x_vec = x.explode()?.i64()?.to_vec();
        let x_vec: Vec<TimeStamp> = x_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| TimeStamp::from(x))
            .collect();
        result_vec.push(x_vec);
    }
    Ok(result_vec)
}
