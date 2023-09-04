use crate::data::data_io::{DeviceId, Link, TimeStamp};
use crate::data::{data_io, stream_io};
use crate::utils::config::{LinkFiles, NetworkSettings, TraceFlags};
use crate::utils::constants::{
    COL_BASE_STATIONS, COL_ROADSIDE_UNITS, COL_RSU_ID, COL_VEHICLES, COL_VEHICLE_ID, STREAM_TIME,
};
use krabmaga::hashbrown::HashMap;
use log::{debug, info};
use std::path::{Path, PathBuf};

pub(crate) struct Vanet {
    pub(crate) config_path: PathBuf,
    pub(crate) trace_flags: TraceFlags,
    pub(crate) network_settings: NetworkSettings,
    pub(crate) link_files: LinkFiles,
    pub(crate) mesh_links: MeshLinks,
    pub(crate) infra_links: InfraLinks,
    pub(crate) step: u64,
}

pub(crate) struct MeshLinks {
    pub(crate) v2v_links: HashMap<TimeStamp, HashMap<DeviceId, Link>>,
    pub(crate) rsu2rsu_links: HashMap<TimeStamp, HashMap<DeviceId, Link>>,
    pub(crate) v2rsu_links: HashMap<TimeStamp, HashMap<DeviceId, Link>>,
}

pub(crate) struct InfraLinks {
    pub(crate) v2bs_links: HashMap<TimeStamp, HashMap<DeviceId, Link>>,
    pub(crate) rsu2bs_links: HashMap<TimeStamp, HashMap<DeviceId, Link>>,
    pub(crate) bs2c_links: HashMap<DeviceId, DeviceId>,
}

impl MeshLinks {
    pub(crate) fn new() -> Self {
        Self {
            v2v_links: HashMap::new(),
            rsu2rsu_links: HashMap::new(),
            v2rsu_links: HashMap::new(),
        }
    }
}

impl InfraLinks {
    pub(crate) fn new() -> Self {
        Self {
            v2bs_links: HashMap::new(),
            rsu2bs_links: HashMap::new(),
            bs2c_links: HashMap::new(),
        }
    }
}

impl Vanet {
    pub(crate) fn new(
        config_path: &PathBuf,
        link_files: &LinkFiles,
        network_settings: &NetworkSettings,
        trace_flags: &TraceFlags,
    ) -> Self {
        let mesh_links = MeshLinks::new();
        let infra_links = InfraLinks::new();
        Self {
            config_path: config_path.clone(),
            network_settings: network_settings.clone(),
            trace_flags: trace_flags.clone(),
            link_files: link_files.clone(),
            mesh_links,
            infra_links,
            step: 0,
        }
    }

    pub(crate) fn init(&mut self) {
        info!("Initializing VANET...");
        self.infra_links.bs2c_links = self.read_bs2c_links();
        self.infra_links.rsu2bs_links = self.read_rsu2bs_links();
        self.mesh_links.rsu2rsu_links = self.read_rsu2rsu_links();
        self.mesh_links.v2v_links = self.read_v2v_links();
        self.mesh_links.v2rsu_links = self.read_v2rsu_links();
        self.infra_links.v2bs_links = self.read_v2bs_links();
    }

    fn stream_links_between_devices(
        &self,
        links_file: PathBuf,
        device_column: &str,
        neighbour_column: &str,
    ) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        let starting_time: u64 = self.step;
        let ending_time: u64 = self.step + STREAM_TIME;
        debug!("Streaming links from file: {}", links_file.display());
        debug!("Starting time: {}", starting_time);
        debug!("Ending time: {}", ending_time);
        let links: HashMap<TimeStamp, HashMap<DeviceId, Link>> =
            match stream_io::stream_links_in_interval(
                links_file,
                device_column,
                neighbour_column,
                starting_time,
                ending_time,
            ) {
                Ok(links) => links,
                Err(e) => {
                    panic!("Error while reading links: {}", e);
                }
            };
        return links;
    }

    fn read_all_links(
        &self,
        links_file: PathBuf,
        device_column: &str,
        neighbour_column: &str,
    ) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        let links: HashMap<TimeStamp, HashMap<DeviceId, Link>> =
            match data_io::read_all_links(links_file, device_column, neighbour_column) {
                Ok(bs2c_links) => bs2c_links,
                Err(e) => {
                    panic!("Error while reading links: {}", e);
                }
            };
        return links;
    }

    fn read_bs2c_links(&self) -> HashMap<DeviceId, DeviceId> {
        info!("Reading base station <-> controller links...");
        let bs2c_links_file = Path::new(&self.config_path).join(&self.link_files.bs2c_links);
        if bs2c_links_file.exists() == false {
            panic!("Base stations to controller link file is not found.");
        }

        let bs2c_links: HashMap<DeviceId, DeviceId> = data_io::read_bs2c_links(bs2c_links_file);
        return bs2c_links;
    }

    fn read_rsu2bs_links(&self) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        info!("Reading RSU <-> base station links...");
        let rsu2bs_links_file = Path::new(&self.config_path).join(&self.link_files.rsu2bs_links);
        if rsu2bs_links_file.exists() == false {
            panic!("RSU to base station file is not found.");
        }

        if self.trace_flags.roadside_unit == true {
            return self.stream_links_between_devices(
                rsu2bs_links_file,
                COL_RSU_ID,
                COL_BASE_STATIONS,
            );
        } else {
            return self.read_all_links(rsu2bs_links_file, COL_RSU_ID, COL_BASE_STATIONS);
        };
    }

    fn read_rsu2rsu_links(&self) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        info!("Reading RSU <-> RSU links...");
        let rsu2rsu_links_file = Path::new(&self.config_path).join(&self.link_files.rsu2rsu_links);
        if rsu2rsu_links_file.exists() == false {
            panic!("RSU to RSU links file is not found.");
        }

        if self.trace_flags.roadside_unit == true {
            return self.stream_links_between_devices(
                rsu2rsu_links_file,
                COL_RSU_ID,
                COL_ROADSIDE_UNITS,
            );
        } else {
            return self.read_all_links(rsu2rsu_links_file, COL_RSU_ID, COL_ROADSIDE_UNITS);
        };
    }

    fn read_v2v_links(&self) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        info!("Reading Vehicle <-> Vehicle links...");
        let v2v_links_file = Path::new(&self.config_path).join(&self.link_files.v2v_links);
        if v2v_links_file.exists() == false {
            panic!("Vehicle to vehicle links file is not found.");
        }

        if self.trace_flags.vehicle == true {
            return self.stream_links_between_devices(v2v_links_file, COL_VEHICLE_ID, COL_VEHICLES);
        } else {
            return self.read_all_links(v2v_links_file, COL_VEHICLE_ID, COL_VEHICLES);
        };
    }

    fn read_v2rsu_links(&self) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        info!("Reading Vehicle <-> RSU links...");
        let v2rsu_links_file = Path::new(&self.config_path).join(&self.link_files.v2rsu_links);
        if v2rsu_links_file.exists() == false {
            panic!("Vehicle to RSU links file is not found.");
        }

        if self.trace_flags.vehicle == true {
            return self.stream_links_between_devices(
                v2rsu_links_file,
                COL_VEHICLE_ID,
                COL_ROADSIDE_UNITS,
            );
        } else {
            return self.read_all_links(v2rsu_links_file, COL_VEHICLE_ID, COL_ROADSIDE_UNITS);
        };
    }

    fn read_v2bs_links(&self) -> HashMap<TimeStamp, HashMap<DeviceId, Link>> {
        info!("Reading Vehicle <-> Base Station links...");
        let v2bs_links_file = Path::new(&self.config_path).join(&self.link_files.v2bs_links);
        if v2bs_links_file.exists() == false {
            panic!("Vehicle to Base Station links file is not found.");
        }

        if self.trace_flags.vehicle == true {
            return self.stream_links_between_devices(
                v2bs_links_file,
                COL_VEHICLE_ID,
                COL_BASE_STATIONS,
            );
        } else {
            return self.read_all_links(v2bs_links_file, COL_VEHICLE_ID, COL_BASE_STATIONS);
        };
    }
}
