use crate::common::columns::COL_TIME_STEP;
use pavenet_config::config::base::TimeStamp;
use polars::prelude::{col, lit, LazyFrame, PolarsError, PolarsResult, ScanArgsParquet};
use polars_core::error::ErrString;
use polars_core::prelude::DataFrame;
use polars_io::{prelude, SerReader};
use std::path::PathBuf;

pub(crate) fn stream_parquet_in_interval(
    file_name: &PathBuf,
    interval_begin: TimeStamp,
    interval_end: TimeStamp,
) -> PolarsResult<DataFrame> {
    let args = ScanArgsParquet::default();
    LazyFrame::scan_parquet(file_name, args)?
        .filter(col(COL_TIME_STEP).gt(lit(interval_begin)))
        .filter(col(COL_TIME_STEP).lt(lit(interval_end)))
        .collect()
}

pub fn read_file(file_name: &PathBuf) -> PolarsResult<DataFrame> {
    let file_extension = file_name.extension().unwrap();
    if file_extension == "csv" {
        return prelude::CsvReader::from_path(file_name)?
            .has_header(true)
            .finish();
    }
    if file_extension == "parquet" {
        let args = ScanArgsParquet::default();
        return LazyFrame::scan_parquet(file_name, args)?.collect();
    }
    return Err(PolarsError::InvalidOperation(ErrString::from(
        "File extension not supported".to_string(),
    )));
}
