use crate::model::{Model, ModelSettings};
use log::error;
use pavenet_core::entity::NodeClass;
use pavenet_core::radio::{DLink, OutgoingStats};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct SelectorSettings {
    pub target_class: NodeClass,
    pub strategy: Strategy,
}

#[derive(Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
pub struct Strategy {
    pub name: String,
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl ModelSettings for SelectorSettings {}

#[derive(Clone, Debug, Default)]
pub enum Selector {
    #[default]
    None,
    All,
    Nearest(NearestSelector),
    Random(RandomSelector),
    MinimumNeighbors(MinimumNeighborSelector),
    MinimumData(MinimumDataSelector),
}

impl Model for Selector {
    type Settings = SelectorSettings;

    fn with_settings(settings: &SelectorSettings) -> Self {
        match settings.strategy.name.to_lowercase().as_str() {
            "none" => Selector::None,
            "all" => Selector::All,
            "nearest" => return Selector::Nearest(NearestSelector::new(settings)),
            "random" => return Selector::Random(RandomSelector::new(settings)),
            "min_neighbors" => return Selector::Random(RandomSelector::new(settings)),
            "min_data" => return Selector::Random(RandomSelector::new(settings)),
            _ => {
                error!("Only basic, nearest, random, min_neighbors and min_data neighbors are supported");
                panic!("Unsupported selector type {}.", settings.strategy.name);
            }
        }
    }
}

impl Selector {
    pub fn do_selection(
        &self,
        links: Vec<DLink>,
        stats: &Vec<Option<&OutgoingStats>>,
    ) -> Vec<DLink> {
        if links.len() == 1 {
            return links;
        }

        match self {
            Selector::None => vec![],
            Selector::All => links,
            Selector::Random(selector) => selector.select_link(links),
            Selector::Nearest(selector) => selector.select_link(links),
            Selector::MinimumNeighbors(selector) => selector.select_link(links, stats),
            Selector::MinimumData(selector) => selector.select_link(links, stats),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NearestSelector {
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl NearestSelector {
    fn new(settings: &SelectorSettings) -> Self {
        Self {
            link_count: settings.strategy.link_count,
            dist_threshold: settings.strategy.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<DLink>) -> Vec<DLink> {
        return links[0..1].to_vec();
    }
}

#[derive(Clone, Debug, Default)]
pub struct RandomSelector {
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl RandomSelector {
    fn new(settings: &SelectorSettings) -> Self {
        Self {
            link_count: settings.strategy.link_count,
            dist_threshold: settings.strategy.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<DLink>) -> Vec<DLink> {
        return links;
    }
}
#[derive(Clone, Debug, Default)]
pub struct MinimumNeighborSelector {
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl MinimumNeighborSelector {
    fn new(settings: &SelectorSettings) -> Self {
        Self {
            link_count: settings.strategy.link_count,
            dist_threshold: settings.strategy.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<DLink>, stats: &Vec<Option<&OutgoingStats>>) -> Vec<DLink> {
        return links;
    }
}

#[derive(Clone, Debug, Default)]
pub struct MinimumDataSelector {
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl MinimumDataSelector {
    fn new(settings: &SelectorSettings) -> Self {
        Self {
            link_count: settings.strategy.link_count,
            dist_threshold: settings.strategy.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<DLink>, stats: &Vec<Option<&OutgoingStats>>) -> Vec<DLink> {
        return links;
    }
}
