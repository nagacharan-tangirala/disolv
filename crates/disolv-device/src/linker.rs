use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::hashbrown::HashMap;
use disolv_core::model::BucketModel;
use disolv_input::links::{LinkMap, LinkReader};
use disolv_models::device::types::DeviceType;
use disolv_models::net::radio::DLink;
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub target_type: DeviceType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct Linker {
    pub source_type: DeviceType,
    pub target_type: DeviceType,
    pub reader: LinkReader,
    pub is_static: bool,
    #[builder(default)]
    pub links: LinkMap,
    #[builder(default)]
    pub link_cache: HashMap<AgentId, Vec<DLink>>,
}

impl Linker {
    pub fn links_of(&mut self, node_id: AgentId) -> Option<Vec<DLink>> {
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
