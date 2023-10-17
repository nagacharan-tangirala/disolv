use crate::columns::TIME_STEP;

use pavenet_recipe::times::ts::TimeS;
use polars::error::ErrString;
use polars::frame::DataFrame;
use polars::io::SerReader;
use polars::prelude::{col, lit, CsvReader, LazyFrame, PolarsError, PolarsResult, ScanArgsParquet};
use std::path::PathBuf;

pub(crate) fn stream_parquet_in_interval(
    file_name: &PathBuf,
    interval_begin: TimeS,
    interval_end: TimeS,
) -> PolarsResult<DataFrame> {
    let args = ScanArgsParquet::default();
    LazyFrame::scan_parquet(file_name, args)?
        .filter(col(TIME_STEP).gt(lit(interval_begin.as_u64())))
        .filter(col(TIME_STEP).lt(lit(interval_end.as_u64())))
        .collect()
}

pub fn read_file(file_name: &PathBuf) -> PolarsResult<DataFrame> {
    let file_extension = file_name.extension().unwrap();
    if file_extension == "csv" {
        return CsvReader::from_path(file_name)?.has_header(true).finish();
    }
    if file_extension == "parquet" {
        let args = ScanArgsParquet::default();
        return LazyFrame::scan_parquet(file_name, args)?.collect();
    }
    return Err(PolarsError::InvalidOperation(ErrString::from(
        "File extension not supported".to_string(),
    )));
}
