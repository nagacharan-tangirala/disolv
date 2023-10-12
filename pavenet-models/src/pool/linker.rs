use crate::model::PoolModel;
use crate::node::payload::Payload;
use hashbrown::HashMap;
use pavenet_core::enums::{NodeType, TransferMode};
use pavenet_core::structs::Link;
use pavenet_core::types::{NodeId, Order, TimeStamp};
use pavenet_input::input::links::{LinkMap, LinkReaderType, LinksFetcher};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub transfer_mode: TransferMode,
    pub target_type: NodeType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}

pub struct Linker {
    pub linker_settings: LinkerSettings,
    pub reader: LinkReaderType,
    pub links: LinkMap,
    pub link_cache: HashMap<NodeId, Link>,
    pub uplink: HashMap<NodeId, Vec<Payload>>,
    pub downlink: HashMap<NodeId, Vec<Payload>>,
}

impl Linker {
    pub fn new(link_config: LinkerSettings, reader: LinkReaderType) -> Self {
        Self {
            linker_settings: link_config,
            reader,
            links: HashMap::new(),
            link_cache: HashMap::new(),
            uplink: HashMap::new(),
            downlink: HashMap::new(),
        }
    }
}

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

#[derive(Default)]
pub struct NodeLinks {
    target_type_links: Vec<(NodeType, Linker)>,
}

impl NodeLinks {
    pub fn new(target_type_links: Vec<(NodeType, Linker)>) -> Self {
        Self { target_type_links }
    }

    pub fn links_for(&mut self, node_id: NodeId, target_type: NodeType) -> Link {
        self.linker_for(target_type)
            .link_cache
            .remove(&node_id)
            .unwrap_or_default()
    }

    fn linker_for(&mut self, target_type: NodeType) -> &mut Linker {
        self.target_type_links
            .iter_mut()
            .find(|(node_type, _)| *node_type == target_type)
            .map(|(_, links)| links)
            .unwrap_or_else(|| panic!("No links found for target node type: {:?}", target_type))
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
