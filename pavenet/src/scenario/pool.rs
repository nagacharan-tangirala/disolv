use crate::scenario::device::Device;
use crate::scenario::episode::{Episode, EpisodeInfo, EpisodeType, NodeChanges, NodeScope};
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use log::error;
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_engine::node::pool::NodePool;
use pavenet_models::model::PoolModel;
use pavenet_models::pool::linker::Linker;
use pavenet_models::pool::space::Space;
use rand::Rng;

pub struct Devices {
    devices: HashMap<NodeId, Device>,
    pub episode: Episode,
    pub to_add: Vec<(NodeId, TimeStamp)>,
    pub to_pop: Vec<NodeId>,
    pub space: Space,
    pub linker: Linker,
}

impl Devices {
    pub fn new(space: Space, linker: Linker, episode: Episode) -> Self {
        Self {
            space,
            linker,
            episode,
            ..Default::default()
        }
    }

    fn apply_episodes(&mut self, step: TimeStamp) {
        if !self.episode.has_episodes_at(step) {
            return;
        }

        let mut episode_list = self.episode.episodes_at(step);
        let mut restore_episodes = Vec::new();
        for episode_info in episode_list.drain(..) {
            let node_config = match episode_info.node_config {
                Some(ref node_settings) => node_settings.clone(),
                None => panic!("No node config found"),
            };
            let node_list = self.episode.filter_nodes(&node_config);
            let device = match self.devices.get_mut(&node_list[0]) {
                Some(device) => device,
                None => panic!("No device found"),
            };
            match episode_info.episode_type {
                EpisodeType::Persistent => {
                    let duration = match episode_info.duration {
                        Some(duration) => duration,
                        None => panic!("No duration found"),
                    };
                    let reset_ts = step + duration;
                    let mut restore = self.episode.get_restore(device, &episode_info, reset_ts);
                    restore.node_config.node_scope = NodeScope::Include(node_list.clone());
                    restore_episodes.push(restore);
                    self.consume_episode(&node_list, &episode_info);
                }
                EpisodeType::Temporary => self.consume_episode(&node_list, &episode_info),
            }
        }
        self.episode.add_episodes(restore_episodes);
    }

    fn consume_episode(&mut self, node_list: &Vec<NodeId>, episode_info: &EpisodeInfo) {
        for node_id in node_list.iter() {
            let mut device = match self.devices.get_mut(node_id) {
                Some(device) => device,
                None => panic!("No device found"),
            };
            match episode_info.model_changes {
                Some(ref model_changes) => device.models.update_models(model_changes),
                None => {}
            }
            match episode_info.node_changes {
                Some(ref node_changes) => self.apply_node_changes(&mut device, node_changes),
                None => {}
            }
        }
    }

    fn apply_node_changes(&mut self, device: &mut Device, node_changes: &NodeChanges) {
        self.episode.remove_from_map(&device.node_info);
        device.apply_node_changes(node_changes);
        self.episode.add_to_map(&device.node_info);
    }
}

impl NodePool for Devices {
    fn init(&mut self, schedule: &mut Schedule) {
        self.space.init(TimeStamp::from(schedule.step));
        self.linker.init(TimeStamp::from(schedule.step));
        let mut by_type_and_class = HashMap::new();
        for device in self.devices.values() {
            by_type_and_class
                .entry(device.node_info.node_type)
                .or_insert_with(HashMap::new)
                .entry(device.node_info.node_class)
                .or_insert_with(Vec::new)
                .push(device.node_info.id);
        }
        self.episode.init(by_type_and_class);
    }

    fn before_step(&mut self, step: TimeStamp) {
        self.space.refresh_cache(step);
        self.linker.refresh_cache(step);
        self.apply_episodes(step);
    }

    fn update(&mut self, step: TimeStamp) {
        todo!()
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        todo!()
    }

    fn streaming_step(&mut self, step: TimeStamp) {
        self.space.stream_data(step);
    }
}
