use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use polars_core::prelude::Series;
use std::slice;

pub(crate) fn convert_series_to_node_ids(
    series: &Series,
) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<NodeId> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| NodeId::from(x))
        .collect::<Vec<NodeId>>();
    return Ok(option_vec_to_vec);
}

pub(crate) fn convert_series_to_timestamps(
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

pub(crate) fn convert_series_to_integer_vector(
    series: &Series,
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
    let option_vec_to_vec: Vec<i64> = series_to_option_vec.iter().filter_map(|x| *x).collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn convert_series_to_floating_vector(
    series: &Series,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
    let option_vec_to_vec: Vec<f32> = series_to_option_vec
        .iter()
        .filter_map(|x| *x)
        .map(|x| x as f32)
        .collect();
    return Ok(option_vec_to_vec);
}

pub(crate) fn convert_string_to_integer_vector(
    input_str: &str,
) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let input_str = input_str.replace("\"", "");
    let mut output_vec: Vec<NodeId> = Vec::new();
    let split_str: Vec<&str> = input_str.split(" ").collect();
    for s in split_str.iter() {
        output_vec.push(s.parse::<NodeId>()?);
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

pub(crate) fn convert_list_series_to_vector_node_ids(
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

pub(crate) fn convert_list_series_to_vector_floats(
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

pub(crate) fn convert_list_series_to_vector_integers(
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

pub(crate) fn convert_list_series_to_vector_timestamps(
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
