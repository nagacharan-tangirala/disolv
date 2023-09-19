use crate::device::base_station::BaseStation;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::reader::activation::{DeviceId, TimeStamp};
use crate::utils::config::DeviceType;
use crate::utils::dyn_config::{EpisodeInfo, ResetEpisodeInfo};
use krabmaga::engine::schedule::Schedule;
use krabmaga::hashbrown::HashMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct EpisodeUpdater {
    episodes: HashMap<TimeStamp, Vec<EpisodeInfo>>,
    reset_episodes: HashMap<TimeStamp, Vec<ResetEpisodeInfo>>,
}

impl EpisodeUpdater {
    pub(crate) fn init(&mut self, episode_list: &Vec<EpisodeInfo>) {
        for parameter_set in episode_list.iter() {
            let time_stamp = parameter_set.time_stamp;
            self.episodes.entry(time_stamp).or_insert_with(Vec::new);
            self.episodes.entry(time_stamp).and_modify(|v| {
                v.push(parameter_set.clone());
            });
        }
    }

    pub(crate) fn has_episode(&self, time_stamp: &TimeStamp) -> bool {
        self.episodes.contains_key(&time_stamp)
    }

    pub(crate) fn setup_episodes(&mut self, schedule: &mut Schedule) {
        let ts_episodes = match self.episodes.remove(&schedule.step) {
            Some(ps) => ps,
            None => return,
        };

        for episode in ts_episodes.into_iter() {
            let device_type = match episode.device_type {
                Some(device_type) => device_type,
                None => panic!("Device type must be specified if device ID is given."),
            };
            match device_type {
                DeviceType::Vehicle => self.set_episode_to_vehicles(&episode, schedule),
                DeviceType::RSU => self.set_episode_to_rsus(&episode, schedule),
                DeviceType::BaseStation => self.set_episode_to_bs(&episode, schedule),
                _ => panic!("Device type not supported."),
            }
        }
    }

    fn set_episode_to_vehicles(&mut self, episode: &EpisodeInfo, schedule: &mut Schedule) {
        let empty_device_list: Vec<DeviceId> = Vec::new();
        let device_list = match &episode.device_list {
            Some(device_list) => device_list,
            None => &empty_device_list,
        };
        let device_class = match episode.device_class {
            Some(device_class) => device_class,
            None => panic!("Device class must be specified if device ID is given."),
        };

        let mut vehicle_agents = schedule.get_all_events();
        for veh_agent in vehicle_agents.iter_mut() {
            if let Some(vehicle) = veh_agent.downcast_mut::<Vehicle>() {
                if device_list.len() > 0 && !device_list.contains(&vehicle.id) {
                    continue;
                }
                if vehicle.device_class != device_class {
                    continue;
                }
                vehicle.setup_episode(episode.clone());
            }
        }
    }

    fn set_episode_to_rsus(&mut self, episode: &EpisodeInfo, schedule: &mut Schedule) {
        let empty_device_list: Vec<DeviceId> = Vec::new();
        let device_list = match &episode.device_list {
            Some(device_list) => device_list,
            None => &empty_device_list,
        };
        let device_class = match episode.device_class {
            Some(device_class) => device_class,
            None => panic!("Device class must be specified if device ID is given."),
        };

        let mut rsu_agents = schedule.get_all_events();
        for rsu_agent in rsu_agents.iter_mut() {
            if let Some(rsu) = rsu_agent.downcast_mut::<RoadsideUnit>() {
                if device_list.len() > 0 && !device_list.contains(&rsu.id) {
                    continue;
                }
                if rsu.device_class != device_class {
                    continue;
                }
                rsu.setup_episode(&episode);
            }
        }
    }

    fn set_episode_to_bs(&mut self, episode: &EpisodeInfo, schedule: &mut Schedule) {
        let empty_device_list: Vec<DeviceId> = Vec::new();
        let device_list = match &episode.device_list {
            Some(device_list) => device_list,
            None => &empty_device_list,
        };
        let device_class = match episode.device_class {
            Some(device_class) => device_class,
            None => panic!("Device class must be specified if device ID is given."),
        };

        let mut bs_agents = schedule.get_all_events();
        for bs_agent in bs_agents.iter_mut() {
            if let Some(bs) = bs_agent.downcast_mut::<BaseStation>() {
                if device_list.len() > 0 && !device_list.contains(&bs.id) {
                    continue;
                }
                if bs.device_class != device_class {
                    continue;
                }
                bs.setup_episode(&episode);
            }
        }
    }
}
