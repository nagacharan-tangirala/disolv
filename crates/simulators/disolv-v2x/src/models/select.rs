use log::error;
use serde::Deserialize;

use disolv_core::agent::AgentClass;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::Link;
use disolv_models::device::models::LinkSelect;
use disolv_models::net::radio::{CommStats, LinkProperties};

#[derive(Deserialize, Debug, Clone)]
pub struct SelectorSettings {
    pub target_class: AgentClass,
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
        match settings.name.to_lowercase().as_str() {
            "none" => Selector::None,
            "all" => Selector::All,
            "nearest" => Selector::Nearest(NearestSelector::new(settings)),
            "random" => Selector::Random(RandomSelector::new(settings)),
            "min_neighbors" => Selector::Random(RandomSelector::new(settings)),
            "min_data" => Selector::Random(RandomSelector::new(settings)),
            _ => {
                error!("Only basic, nearest, random, min_neighbors and min_data neighbors are supported");
                panic!("Unsupported selector type {}.", settings.name);
            }
        }
    }
}

impl LinkSelect<LinkProperties> for Selector {
    fn select_link(
        &self,
        links: Vec<Link<LinkProperties>>,
        stats: &Vec<&CommStats>,
    ) -> Vec<Link<LinkProperties>> {
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
            link_count: settings.link_count,
            dist_threshold: settings.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<Link<LinkProperties>>) -> Vec<Link<LinkProperties>> {
        links[0..1].to_vec()
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
            link_count: settings.link_count,
            dist_threshold: settings.dist_threshold,
        }
    }

    fn select_link(&self, links: Vec<Link<LinkProperties>>) -> Vec<Link<LinkProperties>> {
        links
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
            link_count: settings.link_count,
            dist_threshold: settings.dist_threshold,
        }
    }

    fn select_link(
        &self,
        links: Vec<Link<LinkProperties>>,
        stats: &Vec<&CommStats>,
    ) -> Vec<Link<LinkProperties>> {
        links
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
            link_count: settings.link_count,
            dist_threshold: settings.dist_threshold,
        }
    }

    fn select_link(
        &self,
        links: Vec<Link<LinkProperties>>,
        stats: &Vec<&CommStats>,
    ) -> Vec<Link<LinkProperties>> {
        links
    }
}
