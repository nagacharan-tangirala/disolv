use crate::d_model::BucketModel;
use pavenet_core::bucket::TimeS;
use pavenet_core::entity::id::NodeId;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::link::DLinkOptions;
use pavenet_engine::hashbrown::HashMap;
use pavenet_input::links::data::{LinkMap, LinkReader, LinksFetcher};
use serde::Deserialize;
use typed_builder::TypedBuilder;

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

#[derive(Clone, TypedBuilder)]
pub struct Linker {
    pub reader: LinkReader,
    #[builder(default)]
    pub links: LinkMap,
    #[builder(default)]
    pub link_cache: HashMap<NodeId, DLinkOptions>,
}

impl Linker {
    pub fn links_of(&mut self, node_id: NodeId) -> Option<DLinkOptions> {
        self.link_cache.remove(&node_id)
    }
}

impl BucketModel for Linker {
    fn init(&mut self, step: TimeS) {
        self.links = match self.reader {
            LinkReader::File(ref mut reader) => reader.fetch_links_data(step).unwrap_or_default(),
            LinkReader::Stream(ref mut reader) => reader.fetch_links_data(step).unwrap_or_default(),
        };
    }

    fn stream_data(&mut self, step: TimeS) {
        match self.reader {
            LinkReader::Stream(ref mut reader) => {
                self.links = reader.fetch_links_data(step).unwrap_or_default()
            }
            _ => {}
        }
    }

    fn refresh_cache(&mut self, step: TimeS) {
        self.link_cache = self.links.remove(&step).unwrap_or_default();
    }
}
