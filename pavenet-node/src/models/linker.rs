use crate::d_model::BucketModel;
use log::debug;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::link::DLink; 
use pavenet_engine::bucket::TimeS;
use pavenet_engine::entity::NodeId;
use pavenet_engine::hashbrown::HashMap;
use pavenet_input::links::data::{LinkMap, LinkReader};
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
    #[builder(default)]
    pub links: LinkMap,
    #[builder(default)]
    pub link_cache: HashMap<NodeId, Vec<DLink>>,
}

impl Linker {
    pub fn links_of(&mut self, node_id: NodeId) -> Option<Vec<DLink>> {
        debug!(
            "Linker::links_of node {} are {:?}",
            node_id,
            self.link_cache.get(&node_id)
        );
        self.link_cache.remove(&node_id)
    }
}

impl BucketModel for Linker {
    fn init(&mut self, step: TimeS) {
        self.links = match self.reader {
            LinkReader::File(ref mut reader) => reader.read_links_data(step),
            LinkReader::Stream(ref mut reader) => reader.stream_links_data(step),
        };
    }

    fn stream_data(&mut self, step: TimeS) {
        match self.reader {
            LinkReader::Stream(ref mut reader) => self.links = reader.stream_links_data(step),
            _ => {}
        };
    }

    fn refresh_cache(&mut self, step: TimeS) {
        self.link_cache = self.links.remove(&step).unwrap_or_default();
    }
}
