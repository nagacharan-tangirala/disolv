use crate::model::PoolModel;
use crate::node::payload::Payload;
use hashbrown::HashMap;
use pavenet_core::enums::{NodeType, TransferMode};
use pavenet_core::structs::Link;
use pavenet_core::types::{NodeId, Order, TimeStamp};
use pavenet_input::input::links::{LinkMap, LinkReaderType, LinksFetcher};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct LinkConfig {
    pub transfer_mode: TransferMode,
    pub target_device: NodeType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub link_config: Vec<LinkConfig>,
}

pub struct Linker {
    pub link_config: LinkConfig,
    pub links: LinkMap,
    pub reader: LinkReaderType,
    pub link_cache: HashMap<NodeId, Link>,
    pub uplink: HashMap<NodeId, Vec<Payload>>,
    pub downlink: HashMap<NodeId, Vec<Payload>>,
}

impl Linker {}

impl PoolModel for Linker {
    fn init(&mut self, step: TimeStamp) {
        self.links = match self.reader {
            LinkReaderType::File(ref mut reader) => {
                reader.fetch_links_data(step).unwrap_or_default()
            }
            LinkReaderType::Stream(ref mut reader) => {
                reader.fetch_links_data(step).unwrap_or_default()
            }
        };
    }

    fn stream_data(&mut self, step: TimeStamp) {
        match self.reader {
            LinkReaderType::Stream(ref mut reader) => {
                self.links = reader.fetch_links_data(step).unwrap_or_default()
            }
            _ => {}
        }
    }

    fn refresh_cache(&mut self, step: TimeStamp) {
        self.link_cache = self.links.remove(&step).unwrap_or_default();
    }
}

pub struct NodeLinks {
    target_type_links: Vec<(NodeType, Linker)>,
}

impl NodeLinks {
    pub fn new(node_type_links: HashMap<NodeType, Linker>) -> Self {
        let mut target_type_links = Vec::with_capacity(node_type_links.len());
        for (node_type, links) in node_type_links {
            target_type_links.push((node_type, links));
        }
        Self { target_type_links }
    }

    pub fn links_for(&mut self, node_id: NodeId, node_type: NodeType) -> Link {
        self.linker_for(node_type)
            .link_cache
            .remove(&node_id)
            .unwrap_or_default()
    }

    fn linker_for(&mut self, node_type: NodeType) -> &mut Linker {
        self.target_type_links
            .iter_mut()
            .find(|(target_type, _)| *target_type == node_type)
            .map(|(_, links)| links)
            .unwrap_or_else(|| panic!("No links found for node type: {:?}", node_type))
    }
}

impl PoolModel for NodeLinks {
    fn init(&mut self, step: TimeStamp) {
        self.target_type_links.iter_mut().for_each(|(_, links)| {
            links.init(step);
        });
    }

    fn stream_data(&mut self, step: TimeStamp) {
        self.target_type_links.iter_mut().for_each(|(_, links)| {
            links.stream_data(step);
        });
    }

    fn refresh_cache(&mut self, step: TimeStamp) {
        self.target_type_links.iter_mut().for_each(|(_, links)| {
            links.refresh_cache(step);
        });
    }
}
