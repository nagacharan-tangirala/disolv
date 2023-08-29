use polars::prelude::{col, lit, LazyFrame, PolarsResult, ScanArgsParquet};
use polars_core::prelude::DataFrame;
use polars_io::{prelude, SerReader};

// Reads entire data from a file.
// These are small files, so streaming is NOT implemented.
pub struct CsvDataReader {
    file_name: String,
}

// Time stamped data is read from a file in chunks.
// Certain assumptions are made about the data format. These are
// required to be able to separate the data reading aspect into
// a separate module.
// - The data is sorted by time in ascending order.
// - The time column is named "time_step" and is always the first column.
// - The time column is of type u64 and is always in milliseconds.
pub struct ParquetDataReader {
    file_name: String,
}

impl CsvDataReader {
    pub fn new(file_name: &str) -> Self {
        let file_name: String = file_name.to_string();
        Self { file_name }
    }

    pub fn read_data(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let df = prelude::CsvReader::from_path(&self.file_name)?
            .has_header(true)
            .finish()?;
        Ok(df)
    }
}

impl ParquetDataReader {
    pub fn new(file_name: &str) -> Self {
        let file_name: String = file_name.to_string();
        Self { file_name }
    }

    pub fn read_data(&mut self, interval_begin: i64, interval_end: i64) -> PolarsResult<DataFrame> {
        let args = ScanArgsParquet::default();
        let data_df = LazyFrame::scan_parquet(&self.file_name, args)
            .unwrap()
            .filter(col("time").gt(lit(interval_begin)))
            .filter(col("time").lt(lit(interval_end)))
            .collect();
        return data_df;
    }
}
