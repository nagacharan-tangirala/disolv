use crate::reader::activation::TimeStamp;
use crate::reader::{df_handler, files};
use crate::sim::vanet::{MultiLinkMap, SingleLinkMap};
use crate::utils::config::{LinkFiles, TraceFlags};
use crate::utils::constants::{
    COL_BASE_STATIONS, COL_ROADSIDE_UNITS, COL_RSU_ID, COL_VEHICLES, COL_VEHICLE_ID,
};
use krabmaga::hashbrown::HashMap;
use log::{debug, info, warn};
use polars_core::prelude::DataFrame;
use std::path::{Path, PathBuf};

pub(crate) struct LinksReader {
    pub(crate) config_path: PathBuf,
    pub(crate) link_files: LinkFiles,
    pub(crate) step: TimeStamp,
    trace_flags: TraceFlags,
    streaming_interval: TimeStamp,
}

impl LinksReader {
    pub(crate) fn new(
        config_path: &PathBuf,
        link_files: &LinkFiles,
        trace_flags: &TraceFlags,
        streaming_interval: TimeStamp,
    ) -> Self {
        Self {
            config_path: config_path.clone(),
            link_files: link_files.clone(),
            trace_flags: trace_flags.clone(),
            streaming_interval,
            step: 0,
        }
    }

    pub(crate) fn read_bs2c_links(&self) -> SingleLinkMap {
        info!("Reading base station <-> controller links...");
        let bs2c_links_file = Path::new(&self.config_path).join(&self.link_files.bs2c_links);
        if bs2c_links_file.exists() == false {
            panic!("Base stations to controller link file is not found.");
        }

        let bs2c_links_df = match files::read_csv_data(bs2c_links_file) {
            Ok(b2c_links_df) => b2c_links_df,
            Err(e) => {
                panic!("Error while reading the bs2c links data from file: {}", e);
            }
        };

        let bs2c_links_map: SingleLinkMap = match df_handler::prepare_b2c_links(&bs2c_links_df) {
            Ok(b2c_links_map) => b2c_links_map,
            Err(e) => {
                panic!("Error while converting BS2C links DF to hashmap: {}", e);
            }
        };
        return bs2c_links_map;
    }

    pub(crate) fn read_c2c_links(&self) -> SingleLinkMap {
        info!("Reading controller <-> controller links...");
        let c2c_links_file = Path::new(&self.config_path).join(&self.link_files.c2c_links);
        if c2c_links_file.exists() == false {
            info!("Skipping optional controller to controller links file.");
            return HashMap::new();
        }

        let c2c_links_df = match files::read_csv_data(c2c_links_file) {
            Ok(c2c_links_df) => c2c_links_df,
            Err(e) => {
                panic!("Error while reading the c2c links data from file: {}", e);
            }
        };

        let c2c_links_map: SingleLinkMap = match df_handler::prepare_c2c_links(&c2c_links_df) {
            Ok(c2c_links_map) => c2c_links_map,
            Err(_e) => {
                warn!("Optional C2C links file has problems. Continuing.");
                HashMap::new()
            }
        };
        return c2c_links_map;
    }

    pub(crate) fn read_rsu2bs_links(&self) -> HashMap<TimeStamp, MultiLinkMap> {
        info!("Reading RSU <-> base station links...");
        let rsu2bs_links_file = Path::new(&self.config_path).join(&self.link_files.rsu2bs_links);
        if rsu2bs_links_file.exists() == false {
            panic!("RSU to base station file is not found.");
        }

        return if self.trace_flags.roadside_unit == true {
            self.stream_links_between_devices(rsu2bs_links_file, COL_RSU_ID, COL_BASE_STATIONS)
        } else {
            self.read_all_links(rsu2bs_links_file, COL_RSU_ID, COL_BASE_STATIONS)
        };
    }

    pub(crate) fn read_rsu2rsu_links(&self) -> HashMap<TimeStamp, MultiLinkMap> {
        info!("Reading RSU <-> RSU links...");
        let rsu2rsu_links_file = Path::new(&self.config_path).join(&self.link_files.rsu2rsu_links);
        if rsu2rsu_links_file.exists() == false {
            panic!("RSU to RSU links file is not found.");
        }

        return if self.trace_flags.roadside_unit == true {
            self.stream_links_between_devices(rsu2rsu_links_file, COL_RSU_ID, COL_ROADSIDE_UNITS)
        } else {
            self.read_all_links(rsu2rsu_links_file, COL_RSU_ID, COL_ROADSIDE_UNITS)
        };
    }

    pub(crate) fn read_v2v_links(&self) -> HashMap<TimeStamp, MultiLinkMap> {
        info!("Reading Vehicle <-> Vehicle links...");
        let v2v_links_file = Path::new(&self.config_path).join(&self.link_files.v2v_links);
        if v2v_links_file.exists() == false {
            panic!("Vehicle to vehicle links file is not found.");
        }

        return if self.trace_flags.vehicle == true {
            self.stream_links_between_devices(v2v_links_file, COL_VEHICLE_ID, COL_VEHICLES)
        } else {
            self.read_all_links(v2v_links_file, COL_VEHICLE_ID, COL_VEHICLES)
        };
    }

    pub(crate) fn read_v2rsu_links(&self) -> HashMap<TimeStamp, MultiLinkMap> {
        info!("Reading Vehicle <-> RSU links...");
        let v2rsu_links_file = Path::new(&self.config_path).join(&self.link_files.v2rsu_links);
        if v2rsu_links_file.exists() == false {
            panic!("Vehicle to RSU links file is not found.");
        }

        return if self.trace_flags.vehicle == true {
            self.stream_links_between_devices(v2rsu_links_file, COL_VEHICLE_ID, COL_ROADSIDE_UNITS)
        } else {
            self.read_all_links(v2rsu_links_file, COL_VEHICLE_ID, COL_ROADSIDE_UNITS)
        };
    }

    pub(crate) fn read_v2bs_links(&self) -> HashMap<TimeStamp, MultiLinkMap> {
        info!("Reading Vehicle <-> Base Station links...");
        let v2bs_links_file = Path::new(&self.config_path).join(&self.link_files.v2bs_links);
        if v2bs_links_file.exists() == false {
            panic!("Vehicle to Base Station links file is not found.");
        }

        return if self.trace_flags.vehicle == true {
            self.stream_links_between_devices(v2bs_links_file, COL_VEHICLE_ID, COL_BASE_STATIONS)
        } else {
            self.read_all_links(v2bs_links_file, COL_VEHICLE_ID, COL_BASE_STATIONS)
        };
    }

    fn stream_links_between_devices(
        &self,
        links_file: PathBuf,
        device_column: &str,
        neighbour_column: &str,
    ) -> HashMap<TimeStamp, MultiLinkMap> {
        let starting_time: TimeStamp = self.step;
        let ending_time: TimeStamp = self.step + self.streaming_interval;
        debug!("Streaming links from file: {}", links_file.display());
        debug!("Starting time: {}", starting_time);
        debug!("Ending time: {}", ending_time);

        let links_df: DataFrame =
            match files::stream_parquet_in_interval(links_file, starting_time, ending_time) {
                Ok(links_df) => links_df,
                Err(e) => {
                    panic!("Error while streaming links: {}", e);
                }
            };

        let streamed_links: HashMap<TimeStamp, MultiLinkMap> =
            match df_handler::prepare_dynamic_links(&links_df, device_column, neighbour_column) {
                Ok(links) => links,
                Err(e) => {
                    panic!("Error while converting links DF to hashmap: {}", e);
                }
            };

        return streamed_links;
    }

    fn read_all_links(
        &self,
        links_file: PathBuf,
        device_column: &str,
        neighbour_column: &str,
    ) -> HashMap<TimeStamp, MultiLinkMap> {
        let links_df = match files::read_csv_data(links_file) {
            Ok(links_df) => links_df,
            Err(e) => {
                panic!("Error while reading links data from file: {}", e);
            }
        };

        let static_links: HashMap<TimeStamp, MultiLinkMap> =
            match df_handler::prepare_static_links(&links_df, device_column, neighbour_column) {
                Ok(links) => links,
                Err(e) => {
                    panic!("Error while converting links DF to hashmap: {}", e);
                }
            };
        return static_links;
    }
}
