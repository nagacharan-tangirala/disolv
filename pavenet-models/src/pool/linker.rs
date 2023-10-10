use crate::model::PoolModel;
use crate::node::payload::Payload;
use hashbrown::HashMap;
use log::error;
use pavenet_core::enums::{NodeType, TransferMode};
use pavenet_core::structs::Link;
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_input::input::links::{LinkMap, LinkReaderType, LinksFetcher};
use serde::Deserialize;
use typed_builder::TypedBuilder;

pub const LINKER_SIZE: usize = 10;

#[derive(Deserialize, Debug, Clone)]
pub struct LinkConfig {
    pub transfer_mode: TransferMode,
    pub target_device: NodeType,
    pub links_file: String,
    pub range: f32,
    pub is_streaming: bool,
}

#[derive(TypedBuilder)]
pub struct Linker {
    pub name: String,
    pub link_config: Vec<LinkConfig>,
    pub links: LinkMap,
    pub reader: LinkReaderType,
    #[builder(default)]
    pub link_cache: HashMap<NodeId, Link>,
    #[builder(default)]
    pub uplink: HashMap<NodeId, Vec<Payload>>,
    #[builder(default)]
    pub downlink: HashMap<NodeId, Vec<Payload>>,
}

impl PoolModel for Linker {
    fn init(&mut self, step: TimeStamp) {
        self.links = match self.reader {
            LinkReaderType::File(ref mut reader) => match reader.fetch_links_data(step) {
                Ok(map) => map,
                Err(e) => {
                    error!("Error reading map state: {}", e);
                    HashMap::new()
                }
            },
            LinkReaderType::Stream(ref mut reader) => match reader.fetch_links_data(step) {
                Ok(map) => map,
                Err(e) => {
                    error!("Error reading map state: {}", e);
                    HashMap::new()
                }
            },
        };
    }

    fn stream_data(&mut self, step: TimeStamp) {
        match self.reader {
            LinkReaderType::Stream(ref mut reader) => {
                self.links = match reader.fetch_links_data(step) {
                    Ok(map) => map,
                    Err(e) => {
                        error!("Error reading map state: {}", e);
                        HashMap::new()
                    }
                };
            }
            _ => {}
        }
    }

    fn refresh_cache(&mut self, step: TimeStamp) {
        self.link_cache = match self.links.remove(&step) {
            Some(traces) => traces,
            None => HashMap::new(),
        };
    }
}
