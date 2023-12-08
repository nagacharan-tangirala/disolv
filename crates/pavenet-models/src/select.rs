use crate::model::{Model, ModelSettings};
use pavenet_core::radio::{DLink, InDataStats};
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug, Clone, Copy)]
pub enum Strategy {
    #[default]
    Nearest,
    Random,
    LeastNodes,
    LeastData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SelectorSettings {
    pub strategy: Strategy,
}

impl ModelSettings for SelectorSettings {}

#[derive(Clone, Debug, Default)]
pub struct Selector {
    pub strategy: Strategy,
}

impl Model for Selector {
    type Settings = SelectorSettings;

    fn with_settings(settings: &SelectorSettings) -> Self {
        Self {
            strategy: settings.strategy,
        }
    }
}

impl Selector {
    pub fn select_target(&self, link_opts: Vec<DLink>, stats: &Vec<Option<&InDataStats>>) -> DLink {
        if link_opts.is_empty() {
            panic!("No target nodes in link: {:?}", link_opts);
        }
        if link_opts.len() == 1 {
            return link_opts[0].clone();
        }
        match self.strategy {
            Strategy::Random => self.a_random(link_opts),
            Strategy::Nearest => self.nearest(link_opts),
            Strategy::LeastNodes => self.with_least_nodes(link_opts, stats),
            Strategy::LeastData => self.with_least_data(link_opts, stats),
        }
    }

    fn a_random(&self, mut link_opts: Vec<DLink>) -> DLink {
        let random_idx = rand::thread_rng().gen_range(0..link_opts.len());
        link_opts.remove(random_idx)
    }

    fn nearest(&self, mut link_opt: Vec<DLink>) -> DLink {
        let mut selected_link = link_opt.remove(0);
        let mut min_distance = match selected_link.properties.distance {
            Some(distance) => distance,
            None => f32::MAX,
        };
        for link in link_opt.into_iter() {
            let distance = match link.properties.distance {
                Some(distance) => distance,
                None => f32::MAX,
            };
            if distance < min_distance {
                min_distance = distance;
                selected_link = link;
            }
        }
        return selected_link;
    }

    fn with_least_nodes(
        &self,
        mut link_opts: Vec<DLink>,
        stats: &Vec<Option<&InDataStats>>,
    ) -> DLink {
        let mut selected_link = link_opts.remove(0);
        let min_tr_count = match stats[0] {
            Some(stat) => stat.attempted.node_count,
            None => u32::MAX,
        };
        for (link, stat) in link_opts.into_iter().zip(stats.into_iter()) {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().attempted.node_count < min_tr_count {
                selected_link = link;
            }
        }
        return selected_link;
    }

    fn with_least_data(
        &self,
        mut link_opts: Vec<DLink>,
        stats: &Vec<Option<&InDataStats>>,
    ) -> DLink {
        let mut selected_link = link_opts.remove(0);
        let min_data_size = match stats[0] {
            Some(stat) => stat.attempted.data_size,
            None => f32::MAX,
        };
        for (link, stat) in link_opts.into_iter().zip(stats.into_iter()) {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().attempted.data_size < min_data_size {
                selected_link = link;
            }
        }
        return selected_link;
    }
}
