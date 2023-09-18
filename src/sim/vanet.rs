use crate::models::aggregator::InfraPayload;
use crate::models::composer::UplinkPayload;
use crate::models::responder::DownlinkPayload;
use crate::reader::activation::{DeviceId, TimeStamp};
use crate::reader::links::LinksReader;
use crate::utils::config::{LinkFiles, NetworkSettings, TraceFlags};
use krabmaga::hashbrown::HashMap;
use log::{debug, info};
use std::path::PathBuf;

pub(crate) type Link = (Vec<DeviceId>, Vec<f32>); // (neighbour_device_ids, distances)
pub(crate) type SingleLinkMap = HashMap<DeviceId, DeviceId>;
pub(crate) type MultiLinkMap = HashMap<DeviceId, Link>;

pub(crate) struct Vanet {
    pub(crate) network_settings: NetworkSettings,
    pub(crate) mesh_links: MeshLinks,
    pub(crate) infra_links: InfraLinks,
    pub(crate) links_reader: LinksReader,
    pub(crate) step: TimeStamp,
    pub(crate) uplink: UplinkPayloads,
    pub(crate) downlink: DownlinkPayloads,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MeshLinks {
    pub(crate) v2v_links: HashMap<TimeStamp, MultiLinkMap>,
    pub(crate) rsu2rsu_links: HashMap<TimeStamp, MultiLinkMap>,
    pub(crate) v2rsu_links: HashMap<TimeStamp, MultiLinkMap>,
    pub(crate) v2v_link_cache: MultiLinkMap,
    pub(crate) rsu2rsu_link_cache: MultiLinkMap,
    pub(crate) v2rsu_link_cache: MultiLinkMap,
    pub(crate) rsu2v_link_cache: HashMap<DeviceId, Vec<DeviceId>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct InfraLinks {
    pub(crate) v2bs_links: HashMap<TimeStamp, MultiLinkMap>,
    pub(crate) rsu2bs_links: HashMap<TimeStamp, MultiLinkMap>,
    pub(crate) bs2c_links: SingleLinkMap,
    pub(crate) c2c_links: SingleLinkMap,
    pub(crate) v2bs_link_cache: MultiLinkMap,
    pub(crate) rsu2bs_link_cache: MultiLinkMap,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UplinkPayloads {
    pub(crate) v2v_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) v2rsu_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) v2bs_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) rsu2v_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) rsu2bs_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) rsu2rsu_data: HashMap<DeviceId, Vec<UplinkPayload>>,
    pub(crate) bs2c_data: HashMap<DeviceId, InfraPayload>,
    pub(crate) c2c_data: HashMap<DeviceId, InfraPayload>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DownlinkPayloads {
    pub(crate) bs2v_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) rsu2v_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) rsu2rsu_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) v2v_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) c2bs_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) bs2rsu_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) rsu_responses: HashMap<DeviceId, DownlinkPayload>,
    pub(crate) controller_responses: HashMap<DeviceId, DownlinkPayload>,
}

impl Vanet {
    pub(crate) fn new(
        config_path: &PathBuf,
        link_files: &LinkFiles,
        network_settings: &NetworkSettings,
        trace_flags: &TraceFlags,
        streaming_interval: u64,
    ) -> Self {
        let links_reader =
            LinksReader::new(config_path, link_files, trace_flags, streaming_interval);
        Self {
            network_settings: network_settings.clone(),
            links_reader,
            mesh_links: MeshLinks::default(),
            infra_links: InfraLinks::default(),
            step: 0,
            uplink: UplinkPayloads::default(),
            downlink: DownlinkPayloads::default(),
        }
    }

    pub(crate) fn init(&mut self) {
        info!("Initializing VANET...");
        self.infra_links.bs2c_links = self.links_reader.read_bs2c_links();
        self.infra_links.rsu2bs_links = self.links_reader.read_rsu2bs_links();
        self.mesh_links.rsu2rsu_links = self.links_reader.read_rsu2rsu_links();
        self.mesh_links.v2v_links = self.links_reader.read_v2v_links();
        self.mesh_links.v2rsu_links = self.links_reader.read_v2rsu_links();
        self.infra_links.v2bs_links = self.links_reader.read_v2bs_links();
        self.infra_links.c2c_links = self.links_reader.read_c2c_links();
    }

    pub(crate) fn before_step(&mut self, step: TimeStamp) {
        self.step = step;
        self.refresh_v2v_cache();
        self.refresh_rsu2rsu_cache();
        self.refresh_v2rsu_cache();
        self.refresh_v2bs_cache();
        self.refresh_rsu2bs_cache();
    }

    pub(crate) fn after_step(&mut self) {}

    pub(crate) fn refresh_links_data(&mut self, step: u64) {
        info! {"Refreshing links data from files at step {}", step}
        self.step = step;
        self.links_reader.step = step;
        self.infra_links.rsu2bs_links = self.links_reader.read_rsu2bs_links();
        self.mesh_links.rsu2rsu_links = self.links_reader.read_rsu2rsu_links();
        self.mesh_links.v2v_links = self.links_reader.read_v2v_links();
        self.mesh_links.v2rsu_links = self.links_reader.read_v2rsu_links();
        self.infra_links.v2bs_links = self.links_reader.read_v2bs_links();
    }

    fn refresh_v2v_cache(&mut self) {
        self.mesh_links.v2v_link_cache = match self.mesh_links.v2v_links.remove(&self.step) {
            Some(v2v_links) => v2v_links,
            None => HashMap::new(),
        };
    }

    fn refresh_rsu2rsu_cache(&mut self) {
        self.mesh_links.rsu2rsu_link_cache = match self.mesh_links.rsu2rsu_links.remove(&self.step)
        {
            Some(rsu2rsu_links) => rsu2rsu_links,
            None => HashMap::new(),
        };
    }

    fn refresh_v2rsu_cache(&mut self) {
        self.mesh_links.v2rsu_link_cache = match self.mesh_links.v2rsu_links.remove(&self.step) {
            Some(v2rsu_links) => v2rsu_links,
            None => HashMap::new(),
        };
    }

    fn refresh_v2bs_cache(&mut self) {
        self.infra_links.v2bs_link_cache = match self.infra_links.v2bs_links.remove(&self.step) {
            Some(v2bs_links) => v2bs_links,
            None => HashMap::new(),
        };
    }

    fn refresh_rsu2bs_cache(&mut self) {
        self.infra_links.rsu2bs_link_cache = match self.infra_links.rsu2bs_links.remove(&self.step)
        {
            Some(rsu2bs_links) => rsu2bs_links,
            None => HashMap::new(),
        };
    }
}
