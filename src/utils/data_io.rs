use crate::utils::constants::TIME_STEP;
use polars::prelude::{col, lit, LazyFrame, PolarsResult, ScanArgsParquet};
use polars_core::prelude::DataFrame;
use polars_io::{prelude, SerReader};
use std::path::PathBuf;

// Reads entire data from a file.
// These are small files, so streaming is NOT implemented.
pub(crate) struct CsvDataReader {
    file_name: PathBuf,
}

impl CsvDataReader {
    pub(crate) fn new(file_name: PathBuf) -> Self {
        Self { file_name }
    }

    pub(crate) fn read_data(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let df = prelude::CsvReader::from_path(&self.file_name)?
            .has_header(true)
            .finish()?;
        Ok(df)
    }
}

// Time stamped data is read from a file in chunks.
// Certain assumptions are made about the data format. These are
// required to be able to separate the data reading aspect into
// a separate module.
// - The data is sorted by time in ascending order.
// - The time column is named "time_step" and is always the first column.
// - The time column is of type u64 and is always in milliseconds.
// If there is a need to feed data in a different format, a new struct
// is required to handle the data reading.
pub(crate) struct ParquetDataReader {
    file_name: PathBuf,
}

impl ParquetDataReader {
    pub(crate) fn new(file_name: PathBuf) -> Self {
        Self { file_name }
    }

    pub(crate) fn read_data(
        &mut self,
        interval_begin: i64,
        interval_end: i64,
    ) -> PolarsResult<DataFrame> {
        let args = ScanArgsParquet::default();
        let data_df = LazyFrame::scan_parquet(&self.file_name, args)
            .unwrap()
            .filter(col(TIME_STEP).gt(lit(interval_begin)))
            .filter(col(TIME_STEP).lt(lit(interval_end)))
            .collect();
        return data_df;
    }
}
