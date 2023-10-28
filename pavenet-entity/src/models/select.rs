use crate::models::channel::InDataStats;
use log::error;
use pavenet_recipe::link::SelectedLink;
use pavenet_recipe::Link;
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

#[derive(Clone, Debug, Default)]
pub struct Selector {
    pub strategy: Strategy,
}

impl Selector {
    pub fn new(selector_settings: &SelectorSettings) -> Self {
        Self {
            strategy: selector_settings.strategy,
        }
    }

    pub fn select_target(&self, link: Link, stats: &Vec<Option<&InDataStats>>) -> SelectedLink {
        if link.target.is_empty() {
            panic!("No target nodes in link: {:?}", link);
        }
        if link.target.len() == 1 {
            return link.to_selected_link(0);
        }
        match self.strategy {
            Strategy::Random => self.select_random(link),
            Strategy::Nearest => self.select_nearest(link),
            Strategy::LeastNodes => self.select_links_with_least_nodes(link, stats),
            Strategy::LeastData => self.select_links_with_least_data(link, stats),
        }
    }

    fn select_random(&self, link: Link) -> SelectedLink {
        let random_idx = rand::thread_rng().gen_range(0..link.target.len());
        link.to_selected_link(random_idx)
    }

    fn select_nearest(&self, link: Link) -> SelectedLink {
        return match link.distance {
            Some(distances) => {
                let mut min_distance = distances[0];
                let mut min_idx = 0;
                for (idx, distance) in distances.iter().enumerate() {
                    if distance < &min_distance {
                        min_distance = *distance;
                        min_idx = idx;
                    }
                }
                link.to_selected_link(min_idx)
            }
            None => {
                error!("No distances given, selecting first node.");
                link.to_selected_link(0)
            }
        };
    }

    fn select_links_with_least_nodes(
        &self,
        link: Link,
        stats: &Vec<Option<&InDataStats>>,
    ) -> SelectedLink {
        let min_tr_count = u32::MAX;
        let mut selected_idx: Option<usize> = None;
        for (idx, stat) in stats.iter().enumerate() {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().node_count < min_tr_count {
                selected_idx = Some(idx);
            }
        }
        if let Some(idx) = selected_idx {
            return link.to_selected_link(idx);
        }
        return self.select_nearest(link);
    }

    fn select_links_with_least_data(
        &self,
        link: Link,
        stats: &Vec<Option<&InDataStats>>,
    ) -> SelectedLink {
        let min_data = f32::MAX;
        let mut selected_idx: Option<usize> = None;
        for (idx, stat) in stats.iter().enumerate() {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().data_size < min_data {
                selected_idx = Some(idx);
            }
        }
        if let Some(idx) = selected_idx {
            return link.to_selected_link(idx);
        }
        return self.select_nearest(link);
    }
}
