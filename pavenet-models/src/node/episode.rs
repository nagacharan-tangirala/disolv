use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use pavenet_config::config::dynamic::{EpisodeInfo, EpisodeType};
use pavenet_config::config::types::{DeviceId, TimeStamp};

#[derive(Clone, Debug, Default)]
pub struct EpisodeUpdater {
    episodes: HashMap<TimeStamp, Vec<EpisodeInfo>>,
}

impl EpisodeUpdater {
    pub fn init(&mut self, episode_list: &Vec<EpisodeInfo>) {
        for parameter_set in episode_list.iter() {
            let time_stamp = parameter_set.time_stamp;
            self.episodes.entry(time_stamp).or_insert_with(Vec::new);
            self.episodes.entry(time_stamp).and_modify(|v| {
                v.push(parameter_set.clone());
            });
        }
    }

    pub fn has_episode(&self, time_stamp: &TimeStamp) -> bool {
        self.episodes.contains_key(time_stamp)
    }

    pub fn apply_episodes(&mut self, schedule: &mut Schedule) {
        let ts_episodes = match self.episodes.remove(&TimeStamp::from(schedule.step)) {
            Some(ps) => ps,
            None => return,
        };

        for episode in ts_episodes.into_iter() {
            self.apply_episode(&episode, schedule);
            match episode.episode_type {
                EpisodeType::Persistent => {}
                EpisodeType::Temporary => {
                    self.create_reset_episode(&episode);
                }
            }
        }
    }

    fn apply_episode(&mut self, episode: &EpisodeInfo, schedule: &mut Schedule) {
        let empty_device_list: Vec<DeviceId> = Vec::new();
        let device_list = match &episode.device_list {
            Some(device_list) => device_list,
            None => &empty_device_list,
        };
        let mut sim_agents = ;
        for sim_agent in sim_agents.iter_mut() {
            if let Some(network_device) = sim_agent.as_device_mut() {
                if device_list.len() > 0 && !device_list.contains(&network_device.id) {
                    continue;
                }
                network_device.setup_episode(episode.clone());
            }
        }
    }

    fn create_reset_episode(&mut self, episode: &EpisodeInfo) {
        let mut episode_clone = episode.clone();
        let duration = match episode_clone.duration {
            Some(duration) => episode_clone.time_stamp + duration,
            None => episode_clone.time_stamp,
        };
        episode_clone.episode_type = EpisodeType::Persistent;
        episode_clone.duration = None;
    }
}
