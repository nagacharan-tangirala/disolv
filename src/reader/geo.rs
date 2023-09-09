use crate::reader::activation::{DeviceId, TimeStamp};
use crate::reader::{df_handler, df_utils, files};
use crate::sim::field::GeoMap;
use crate::utils::config::{GeoDataFiles, TraceFlags};
use crate::utils::constants::{
    COL_BASE_STATION_ID, COL_CONTROLLER_ID, COL_COORD_X, COL_COORD_Y, COL_RSU_ID, COL_VEHICLE_ID,
};
use krabmaga::hashbrown::HashMap;
use std::path::PathBuf;

pub(crate) struct GeoReader {
    pub(crate) config_path: PathBuf,
    pub(crate) geo_files: GeoDataFiles,
    pub(crate) step: TimeStamp,
    trace_flags: TraceFlags,
    streaming_interval: TimeStamp,
}

impl GeoReader {
    pub(crate) fn new(
        config_path: &PathBuf,
        geo_files: &GeoDataFiles,
        trace_flags: &TraceFlags,
        streaming_interval: TimeStamp,
    ) -> Self {
        Self {
            config_path: config_path.clone(),
            geo_files: geo_files.clone(),
            trace_flags: trace_flags.clone(),
            streaming_interval,
            step: 0,
        }
    }

    pub(crate) fn read_vehicle_geo_data(&self) -> GeoMap {
        let trace_file = self.config_path.join(&self.geo_files.vehicle_geo_data);
        if trace_file.exists() == false {
            panic!("Vehicle trace file is not found.");
        }

        let vehicle_positions = if self.trace_flags.vehicle == true {
            self.stream_device_geo_data(trace_file, COL_VEHICLE_ID)
        } else {
            self.read_entire_geo_data(trace_file, COL_VEHICLE_ID)
        };
        return vehicle_positions;
    }

    pub(crate) fn read_rsu_geo_data(&self) -> GeoMap {
        let trace_file = self.config_path.join(&self.geo_files.rsu_geo_data);
        if trace_file.exists() == false {
            panic!("RSU trace file is not found.");
        }

        let rsu_positions = if self.trace_flags.roadside_unit == true {
            self.stream_device_geo_data(trace_file, COL_RSU_ID)
        } else {
            self.read_entire_geo_data(trace_file, COL_RSU_ID)
        };
        return rsu_positions;
    }

    pub(crate) fn read_bs_geo_data(&self) -> GeoMap {
        let trace_file = self.config_path.join(&self.geo_files.bs_geo_data);
        if trace_file.exists() == false {
            panic!("Base station trace file is not found.");
        }

        let bs_positions = if self.trace_flags.base_station == true {
            self.stream_device_geo_data(trace_file, COL_BASE_STATION_ID)
        } else {
            self.read_entire_geo_data(trace_file, COL_BASE_STATION_ID)
        };
        return bs_positions;
    }

    pub(crate) fn read_controller_geo_data(&self) -> HashMap<DeviceId, (f32, f32)> {
        let trace_file = self.config_path.join(&self.geo_files.controller_geo_data);
        if trace_file.exists() == false {
            panic!("Controller trace file is not found.");
        }

        let controller_positions: HashMap<u64, (f32, f32)> =
            match self.read_controller_positions(trace_file) {
                Ok(controller_positions) => controller_positions,
                Err(e) => {
                    panic!("Error while reading controller positions: {}", e);
                }
            };
        controller_positions
    }

    fn read_controller_positions(
        &self,
        controller_file: PathBuf,
    ) -> Result<HashMap<DeviceId, (f32, f32)>, Box<dyn std::error::Error>> {
        let controller_df = files::read_csv_data(controller_file)?;
        let controller_ids: Vec<DeviceId> =
            df_utils::convert_series_to_integer_vector(&controller_df, COL_CONTROLLER_ID)?;
        let x_positions: Vec<f32> =
            df_utils::convert_series_to_floating_vector(&controller_df, COL_COORD_X)?;
        let y_positions: Vec<f32> =
            df_utils::convert_series_to_floating_vector(&controller_df, COL_COORD_Y)?;

        let mut controller_map: HashMap<DeviceId, (f32, f32)> = HashMap::new();
        for i in 0..controller_ids.len() {
            controller_map.insert(controller_ids[i], (x_positions[i], y_positions[i]));
        }

        return Ok(controller_map);
    }

    fn stream_device_geo_data(&self, trace_file: PathBuf, device_id_column: &str) -> GeoMap {
        let start_interval: TimeStamp = self.step;
        let end_interval: TimeStamp = self.step + self.streaming_interval;
        let geo_data_df =
            match files::stream_parquet_in_interval(trace_file, start_interval, end_interval) {
                Ok(trace_df) => trace_df,
                Err(e) => {
                    panic!("Error while streaming parquet: {}", e);
                }
            };

        let geo_map: GeoMap = match df_handler::prepare_geo_data(&geo_data_df, device_id_column) {
            Ok(trace_map) => trace_map,
            Err(e) => {
                panic!("Error while converting geo data DF to hashmap: {}", e);
            }
        };
        return geo_map;
    }

    fn read_entire_geo_data(&self, trace_file: PathBuf, device_id_column: &str) -> GeoMap {
        let trace_df = match files::read_parquet_data(trace_file) {
            Ok(trace_df) => trace_df,
            Err(e) => {
                panic!("Error while reading trace data from the file: {}", e);
            }
        };

        let trace_map: GeoMap = match df_handler::prepare_geo_data(&trace_df, device_id_column) {
            Ok(trace_map) => trace_map,
            Err(e) => {
                panic!("Error while converting DF to hashmap: {}", e);
            }
        };
        return trace_map;
    }
}
