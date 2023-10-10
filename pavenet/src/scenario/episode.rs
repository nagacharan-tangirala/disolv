use crate::scenario::device::Device;
use hashbrown::HashMap;
use pavenet_core::enums::NodeType;
use pavenet_core::named::class::Class;
use pavenet_core::structs::NodeInfo;
use pavenet_core::types::{NodeId, Order, TimeStamp};
use pavenet_models::node::composer::Composer;
use pavenet_models::node::responder::Responder;
use pavenet_models::node::simplifier::Simplifier;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub enum EpisodeType {
    #[default]
    Persistent,
    Temporary,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct EpisodeInfo {
    pub time_stamp: TimeStamp,
    pub episode_type: EpisodeType,
    pub duration: Option<TimeStamp>,
    pub node_config: NodeConfig,
    pub node_changes: Option<NodeChanges>,
    pub model_changes: Option<ModelChanges>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ModelChanges {
    pub composer: Option<Composer>,
    pub simplifier: Option<Simplifier>,
    pub responder: Option<Responder>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(tag = "node_scope", content = "nodes")]
pub enum NodeScope {
    #[default]
    None,
    All,
    Exclude(Vec<NodeId>),
    Include(Vec<NodeId>),
    Ratio(f32),
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct NodeConfig {
    pub node_type: NodeType,
    pub node_class: Class,
    pub node_scope: NodeScope,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct NodeChanges {
    pub new_node_type: NodeType,
    pub new_node_class: Class,
    pub new_order: Order,
}

#[derive(Clone, Debug, Default)]
pub struct Episode {
    node_id_map: HashMap<NodeType, HashMap<Class, Vec<NodeId>>>,
    episodes: HashMap<TimeStamp, Vec<EpisodeInfo>>,
}

impl Episode {
    pub fn new(episode_list: &Vec<EpisodeInfo>) -> Self {
        let mut episode_map = HashMap::new();
        for parameter_set in episode_list.iter() {
            let time_stamp = parameter_set.time_stamp;
            episode_map.entry(time_stamp).or_insert_with(Vec::new);
            episode_map.entry(time_stamp).and_modify(|v| {
                v.push(parameter_set.clone());
            });
        }
        Self {
            episodes: episode_map,
            ..Default::default()
        }
    }

    pub fn init(&mut self, node_id_map: HashMap<NodeType, HashMap<Class, Vec<NodeId>>>) {
        self.node_id_map = node_id_map;
    }

    pub fn has_episode_at(&self, time_stamp: TimeStamp) -> bool {
        self.episodes.contains_key(&time_stamp)
    }

    pub fn episodes_at(&mut self, time_stamp: TimeStamp) -> Vec<EpisodeInfo> {
        self.episodes.remove(&time_stamp).unwrap_or_default()
    }

    pub fn add_episodes(&mut self, episode_list: Vec<EpisodeInfo>) {
        for episode in episode_list.iter() {
            let time_stamp = episode.time_stamp;
            self.episodes.entry(time_stamp).or_insert_with(Vec::new);
            self.episodes.entry(time_stamp).and_modify(|v| {
                v.push(episode.clone());
            });
        }
    }

    pub fn remove_from_map(&mut self, node_info: &NodeInfo) {
        let node_id = node_info.id;
        self.node_id_map
            .entry(node_info.node_type)
            .and_modify(|map_by_class| {
                map_by_class
                    .entry(node_info.node_class)
                    .and_modify(|node_id_vec| {
                        node_id_vec.retain(|&x| x != node_id);
                    });
            });
    }

    pub fn add_to_map(&mut self, node_info: &NodeInfo) {
        let node_id = node_info.id;
        self.node_id_map
            .entry(node_info.node_type)
            .or_insert_with(HashMap::new)
            .entry(node_info.node_class)
            .or_insert_with(Vec::new)
            .push(node_id);
    }

    pub fn filter_nodes(self, node_config: &NodeConfig) -> Vec<NodeId> {
        let mut valid_nodes: Vec<NodeId> = self
            .node_id_map
            .get(&node_config.node_type)
            .and_then(|v| v.get(&node_config.node_class))
            .unwrap_or_default()
            .clone();
        return match node_config.node_scope {
            NodeScope::All => valid_nodes,
            NodeScope::Exclude(ref exclude_list) => {
                for node_id in exclude_list.iter() {
                    valid_nodes.retain(|&x| x != *node_id);
                }
            }
            NodeScope::Include(ref include_list) => include_list,
            NodeScope::Ratio(ratio) => {
                let mut rng = rand::thread_rng();
                valid_nodes.retain(|_| rng.gen::<f32>() < ratio);
                valid_nodes
            }
            NodeScope::None => Vec::new(),
        };
    }

    pub fn get_restore(
        &mut self,
        device: &mut Device,
        episode_info: &EpisodeInfo,
        reset_ts: TimeStamp,
    ) -> EpisodeInfo {
        let mut restore_episode = EpisodeInfo::default();
        restore_episode.time_stamp = reset_ts;
        restore_episode.episode_type = EpisodeType::Persistent;

        let mut node_config = NodeConfig::default();
        node_config.node_type = episode_info.node_config.node_type;
        node_config.node_class = episode_info.node_config.node_class;
        restore_episode.node_config = node_config;
        if episode_info.node_changes.is_some() {
            restore_episode.node_changes = self.node_changes_to_restore(device);
        }

        if episode_info.model_changes.is_some() {
            let new_models = episode_info.model_changes.clone().unwrap();
            restore_episode.model_changes = Some(device.models.fetch_current_settings(&new_models))
        }
        return restore_episode;
    }

    fn node_changes_to_restore(self, device: &mut Device) -> Option<NodeChanges> {
        let mut node_changes = NodeChanges::default();
        node_changes.new_node_type = device.node_info.node_type;
        node_changes.new_node_class = device.node_info.node_class;
        return Some(node_changes);
    }
}
