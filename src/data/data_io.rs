use crate::data::df_handler::*;
use crate::utils::constants::{CONTROLLER_ID, COORD_X, COORD_Y, TIME_STEP};
use krabmaga::hashbrown::HashMap;
use polars::export::arrow::datatypes::{DataType, Field};
use polars::prelude::{col, lit, LazyFrame, PolarsResult, ScanArgsParquet};
use polars_core::prelude::{DataFrame, Schema, SchemaRef};
use polars_io::{prelude, SerReader};
use std::path::PathBuf;

pub type Activation = (Vec<u64>, Vec<u64>);
pub type Trace = (Vec<u64>, Vec<f32>, Vec<f32>, Vec<f32>);

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

// Time stamped data is read from a parquet file in chunks.
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
        interval_begin: u64,
        interval_end: u64,
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

pub(crate) struct ActivationDataReader;

impl ActivationDataReader {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn read_activation_data(
        &self,
        activations_file: PathBuf,
    ) -> HashMap<u64, Activation> {
        let mut activation_data_reader: CsvDataReader = CsvDataReader::new(activations_file);
        let activation_df = match activation_data_reader.read_data() {
            Ok(activation_df) => activation_df,
            Err(e) => {
                panic!("Error while reading activation data from file: {}", e);
            }
        };

        let mut activation_df_handler = ActivationDFHandler::new(activation_df);
        return match activation_df_handler.prepare_device_activations() {
            Ok(activation_map) => activation_map,
            Err(e) => {
                panic!("Error while converting activation DF to hashmap: {}", e);
            }
        };
    }
}

pub(crate) struct PositionsReader;

impl PositionsReader {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn read_position_data(
        &self,
        trace_file: PathBuf,
        device_id_column: &str,
        start_interval: u64,
        end_interval: u64,
    ) -> HashMap<u64, Trace> {
        let mut trace_reader: ParquetDataReader = ParquetDataReader::new(trace_file);

        let trace_df = match trace_reader.read_data(start_interval, end_interval) {
            Ok(trace_df) => trace_df,
            Err(e) => {
                panic!("Error while reading the trace data from file: {}", e);
            }
        };

        let mut trace_handler: TraceDFHandler = TraceDFHandler::new(trace_df);
        let trace_map: HashMap<u64, Trace> =
            match trace_handler.prepare_trace_data(device_id_column) {
                Ok(trace_map) => trace_map,
                Err(e) => {
                    panic!("Error while converting DF to hashmap: {}", e);
                }
            };
        return trace_map;
    }

    pub(crate) fn read_controller_positions(
        &self,
        controller_file: PathBuf,
    ) -> Result<HashMap<u64, (f32, f32)>, Box<dyn std::error::Error>> {
        let mut controller_reader: CsvDataReader = CsvDataReader::new(controller_file);
        let controller_df = match controller_reader.read_data() {
            Ok(controller_df) => controller_df,
            Err(e) => {
                panic!("Error while reading the controller data from file: {}", e);
            }
        };

        let controller_ids: Vec<u64> =
            convert_series_to_integer_vector(&controller_df, CONTROLLER_ID)?;
        let x_positions: Vec<f32> = convert_series_to_floating_vector(&controller_df, COORD_X)?;
        let y_positions: Vec<f32> = convert_series_to_floating_vector(&controller_df, COORD_Y)?;

        let mut controller_map: HashMap<u64, (f32, f32)> = HashMap::new();
        for i in 0..controller_ids.len() {
            controller_map.insert(controller_ids[i], (x_positions[i], y_positions[i]));
        }
        return Ok(controller_map);
    }
}

pub(crate) struct LinksReader {
    config_path: PathBuf,
}

impl LinksReader {
    pub(crate) fn new(config_path: &str) -> Self {
        let config_path = PathBuf::from(config_path);
        Self {
            config_path: config_path,
        }
    }

    // pub(crate) fn read_dynamic_links(
    //     &self,
    //     links_file: PathBuf,
    //     links_column: &str,
    // ) -> Result<HashMap<i64, i64>, Box<dyn std::error::Error>> {
    //     let mut links_reader: ParquetDataReader = ParquetDataReader::new(links_file);
    //     let links_df = match links_reader.read_data() {
    //         Ok(controller_df) => controller_df,
    //         Err(e) => {
    //             panic!("Error while reading the controller data from file: {}", e);
    //         }
    //     };
    // }

    pub(crate) fn read_static_links() {}
}