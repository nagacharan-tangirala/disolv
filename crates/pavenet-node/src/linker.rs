use pavenet_core::entity::NodeType;
use pavenet_engine::bucket::TimeS;
use pavenet_engine::hashbrown::HashMap;
use serde::Deserialize;
use typed_builder::TypedBuilder;
use pavenet_core::radio::DLink;

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
        if self.is_static {
            return self.link_cache.get(&node_id).cloned();
        }
        self.link_cache.remove(&node_id)
    }
}

impl BucketModel for Linker {
    fn init(&mut self, step: TimeS) {
        self.links = match self.reader {
            LinkReader::File(ref mut reader) => reader.read_links_data(step),
            LinkReader::Stream(ref mut reader) => reader.stream_links_data(step),
        };
        // Refresh cache for the first time step
        self.link_cache = match self.links.remove(&step) {
            Some(links) => links,
            None => HashMap::new(),
        };
    }

    fn stream_data(&mut self, step: TimeS) {
        match self.reader {
            LinkReader::Stream(ref mut reader) => self.links = reader.stream_links_data(step),
            _ => {}
        };
    }

    fn refresh_cache(&mut self, step: TimeS) {
        if self.is_static {
            return;
        }

        self.link_cache = match self.links.remove(&step) {
            Some(links) => links,
            None => HashMap::new(),
        };
    }
}
