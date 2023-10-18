use hashbrown::HashMap;
use pavenet_input::links::data::{LinkMap, LinkReader};
use pavenet_recipe::link::Link;
use pavenet_recipe::node_info::id::NodeId;
use pavenet_recipe::node_info::kind::NodeType;
use pavenet_recipe::payload::TPayload;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Copy)]
pub enum TransferMode {
    UDT,
    BDT,
}

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
    pub reader: LinkReader,
    pub links: LinkMap,
    pub link_cache: HashMap<NodeId, Link>,
    pub uplink: HashMap<NodeId, Vec<TPayload>>,
    pub downlink: HashMap<NodeId, Vec<TPayload>>,
}

impl Linker {
    pub fn new(link_config: LinkerSettings, reader: LinkReader) -> Self {
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
            LinkReader::File(ref mut reader) => reader.fetch_links_data(step).unwrap_or_default(),
            LinkReader::Stream(ref mut reader) => reader.fetch_links_data(step).unwrap_or_default(),
        };
    }

    fn stream_data(&mut self, step: TimeStamp) {
        match self.reader {
            LinkReader::Stream(ref mut reader) => {
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

    pub fn links_for(&mut self, node_id: NodeId, target_type: &NodeType) -> Link {
        self.linker_for(target_type)
            .link_cache
            .remove(&node_id)
            .unwrap_or_default()
    }

    fn linker_for(&mut self, target_type: &NodeType) -> &mut Linker {
        self.target_type_links
            .iter_mut()
            .find(|(node_type, _)| *node_type == *target_type)
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
