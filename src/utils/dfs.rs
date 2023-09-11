use polars_core::prelude::{DataFrame, PolarsResult, Series};

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
