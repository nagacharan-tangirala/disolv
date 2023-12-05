use log::debug;
use pavenet_core::entity::NodeType;
use pavenet_core::radio::DLink;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::node::NodeId;
use pavenet_input::links::data::{LinkMap, LinkReader};
use pavenet_models::model::BucketModel;
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub target_type: NodeType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct Linker {
    pub reader: LinkReader,
    pub is_static: bool,
    #[builder(default)]
    pub links: LinkMap,
    #[builder(default)]
    pub link_cache: HashMap<NodeId, Vec<DLink>>,
}

impl Linker {
    pub fn links_of(&mut self, node_id: NodeId) -> Option<Vec<DLink>> {
        debug!("Linker::links_of: node_id: {:?}", self.links.len());
        if self.is_static {
            return self.link_cache.get(&node_id).cloned();
        }
        self.link_cache.remove(&node_id)
    }
}

impl BucketModel for Linker {
    fn init(&mut self, step: TimeMS) {
        self.links = match self.reader {
            LinkReader::File(ref mut reader) => reader.read_links_data(step),
            LinkReader::Stream(ref mut reader) => reader.stream_links_data(step),
        };
        // Refresh cache for the first time step
        debug!(
            "Reading links with settings: {} - {}",
            self.is_static, self.reader
        );
        self.link_cache = match self.links.remove(&step) {
            Some(links) => links,
            None => HashMap::new(),
        };
    }

    fn stream_data(&mut self, step: TimeMS) {
        if let LinkReader::Stream(ref mut reader) = self.reader {
            self.links = reader.stream_links_data(step)
        };
    }

    fn before_node_step(&mut self, step: TimeMS) {
        if self.is_static {
            return;
        }

        self.link_cache = match self.links.remove(&step) {
            Some(links) => links,
            None => HashMap::new(),
        };
    }
}
