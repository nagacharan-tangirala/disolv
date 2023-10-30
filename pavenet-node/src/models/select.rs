use pavenet_core::link::{DLink, DLinkOptions};
use pavenet_core::radio::stats::InDataStats;
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

    pub fn select_target(
        &self,
        mut link_opts: DLinkOptions,
        stats: &Vec<Option<&InDataStats>>,
    ) -> DLink {
        if link_opts.is_empty() {
            panic!("No target nodes in link: {:?}", link_opts);
        }
        if link_opts.len() == 1 {
            return link_opts[0];
        }
        match self.strategy {
            Strategy::Random => self.a_random(link_opts),
            Strategy::Nearest => self.nearest(link_opts),
            Strategy::LeastNodes => self.with_least_nodes(link_opts, stats),
            Strategy::LeastData => self.with_least_data(link_opts, stats),
        }
    }

    fn a_random(&self, mut link_opt: DLinkOptions) -> DLink {
        let random_idx = rand::thread_rng().gen_range(0..link_opt.len());
        link_opt.utilize_link_at(random_idx)
    }

    fn nearest(&self, mut link_opt: DLinkOptions) -> DLink {
        let mut selected_link = link_opt.utilize_link_at(0);
        let mut min_distance = match selected_link.properties.distance {
            Some(distance) => distance,
            None => f32::MAX,
        };
        for link in link_opt {
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
        mut link_opt: DLinkOptions,
        stats: &Vec<Option<&InDataStats>>,
    ) -> DLink {
        let min_tr_count = u32::MAX;
        let mut selected_idx: Option<usize> = None;
        for (idx, stat) in stats.iter().enumerate() {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().attempted.node_count < min_tr_count {
                selected_idx = Some(idx);
            }
        }
        if let Some(idx) = selected_idx {
            return link_opt.to_selected_link(idx);
        }
        return self.nearest(link_opt);
    }

    fn with_least_data(&self, link_opt: DLinkOptions, stats: &Vec<Option<&InDataStats>>) -> DLink {
        let min_data = f32::MAX;
        let mut selected_idx: Option<usize> = None;
        for (idx, stat) in stats.iter().enumerate() {
            if stat.is_none() {
                continue;
            }
            if stat.unwrap().attempted.data_size < min_data {
                selected_idx = Some(idx);
            }
        }
        if let Some(idx) = selected_idx {
            return link_opt.to_selected_link(idx);
        }
        return self.nearest(link_opt);
    }
}