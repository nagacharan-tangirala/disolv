use crate::data::data_io::TimeStamp;
use crate::utils::constants::COL_TIME_STEP;
use polars::prelude::{col, lit, LazyFrame, PolarsResult, ScanArgsParquet};
use polars_core::prelude::DataFrame;
use polars_io::{prelude, SerReader};
use std::path::PathBuf;

pub(crate) fn stream_parquet_in_interval(
    file_name: PathBuf,
    interval_begin: TimeStamp,
    interval_end: TimeStamp,
) -> PolarsResult<DataFrame> {
    let args = ScanArgsParquet::default();
    LazyFrame::scan_parquet(file_name, args)?
        .filter(col(COL_TIME_STEP).gt(lit(interval_begin)))
        .filter(col(COL_TIME_STEP).lt(lit(interval_end)))
        .collect()
}

pub(crate) fn read_parquet_data(file_name: PathBuf) -> PolarsResult<DataFrame> {
    let args = ScanArgsParquet::default();
    LazyFrame::scan_parquet(file_name, args)?.collect()
}

pub(crate) fn read_csv_data(file_name: PathBuf) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let df = prelude::CsvReader::from_path(file_name)?
        .has_header(true)
        .finish()?;
    Ok(df)
}
