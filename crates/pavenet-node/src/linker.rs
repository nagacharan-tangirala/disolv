use pavenet_core::entity::NodeType;
use pavenet_core::radio::DLink;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::node::NodeId;
use pavenet_input::links::{LinkMap, LinkReader};
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
    pub source_type: NodeType,
    pub target_type: NodeType,
    pub reader: LinkReader,
    pub is_static: bool,
    #[builder(default)]
    pub links: LinkMap,
    #[builder(default)]
    pub link_cache: HashMap<NodeId, Vec<DLink>>,
}

impl Linker {
    pub fn links_of(&mut self, node_id: NodeId) -> Option<Vec<DLink>> {
        if self.is_static {
            return self.link_cache.get(&node_id).cloned();
        }
        self.link_cache.remove(&node_id)
    }
}

impl BucketModel for Linker {
    fn init(&mut self, step: TimeMS) {
        self.links = self.reader.fetch_links_data(step);
        self.link_cache = self.links.remove(&step).unwrap_or_default();
    }

    fn stream_data(&mut self, step: TimeMS) {
        if self.reader.is_streaming {
            self.links = self.reader.fetch_links_data(step);
        }
    }

    fn before_node_step(&mut self, step: TimeMS) {
        if self.is_static {
            return;
        }
        self.link_cache = self.links.remove(&step).unwrap_or_default();
    }
}
