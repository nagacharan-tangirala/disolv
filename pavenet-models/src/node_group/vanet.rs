use crate::node::composer::Payload;
use hashbrown::HashMap;
use log::error;
use pavenet_config::config::base::{LinkMap, MultiLinkMap, NetworkSettings};
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use pavenet_io::input::links::{LinkMapType, LinkReaderType, LinksFetcher, ReadLinks};
use std::path::PathBuf;

pub type Link = (Vec<NodeId>, Vec<f32>); // (neighbour_device_ids, distances)

enum LinkCacheType {
    Single(LinkMap),
    Multiple(MultiLinkMap),
}

pub struct Vanet {
    pub network_settings: NetworkSettings,
    pub link_map: LinkMapType,
    pub links_reader: LinkReaderType,
    pub link_cache: LinkCacheType,
    pub uplink: HashMap<NodeId, Vec<Payload>>,
    pub downlink: HashMap<NodeId, Vec<Payload>>,
}

impl Vanet {
    pub fn new(
        config_path: &PathBuf,
        network_settings: &NetworkSettings,
        streaming_interval: u64,
    ) -> Self {
        let links_reader = ReadLinks::new(config_path, streaming_interval);
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

    pub fn init(&mut self, step: TimeStamp) {
        self.read_links(&mut self.links_reader, step);
    }

    pub fn before_step(&mut self, step: TimeStamp) {
        self.refresh_link_cache(step);
    }
    
    fn refresh_link_cache(&mut self, step: TimeStamp) {
        self.link_cache = match self.link_map {
            LinkMapType::LinkTraceMap(ref links) => {
                let links_map = match links.get(&step) {
                    Some(links) => links.clone(),
                    None => {
                        error!("No links data found for step {}", step);
                        std::process::exit(1);
                    }
                };
                LinkCacheType::Multiple(links_map)
            }
            _ => {}
        };    
    }
    
    fn read_links(&mut self, link_reader: &mut dyn LinksFetcher, step: TimeStamp) {
        self.link_map = match link_reader.fetch_links_data(step) {
            Ok(links) => links,
            Err(e) => {
                error!("Error reading links data: {}", e);
                std::process::exit(1);
            }
        };
    }

    pub fn after_step(&mut self) {}

    pub fn refresh_links_data(&mut self, step: u64) {
        info! {"Refreshing links data from files at step {}", step}
        self.step = step;
        self.links_reader.step = step;
        self.infra_links.rsu2bs_links = self.links_reader.read_rsu2bs_links();
        self.mesh_links.rsu2rsu_links = self.links_reader.read_rsu2rsu_links();
        self.mesh_links.v2v_links = self.links_reader.read_v2v_links();
        self.mesh_links.v2rsu_links = self.links_reader.read_v2rsu_links();
        self.infra_links.v2bs_links = self.links_reader.read_v2bs_links();
    }
}
