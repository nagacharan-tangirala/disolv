use log::error;
use serde::Deserialize;

use disolv_core::agent::AgentClass;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::Link;
use disolv_models::device::models::LinkSelect;
use disolv_models::net::radio::{CommStats, LinkProperties};

#[derive(Deserialize, Debug, Clone)]
pub struct LinkSelectionSettings {
    pub target_class: AgentClass,
    pub name: String,
    pub link_count: Option<u32>,
    pub dist_threshold: Option<f32>,
}

impl ModelSettings for LinkSelectionSettings {}

#[derive(Debug, Clone)]
pub(crate) enum LinkSelector {
    All,
    Nearest(NearestLink),
}

impl Model for LinkSelector {
    type Settings = LinkSelectionSettings;

    fn with_settings(settings: &LinkSelectionSettings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "all" => LinkSelector::All,
            "nearest" => LinkSelector::Nearest(NearestLink::new(settings)),
            _ => {
                error!("Only all and nearest link selector is supported");
                panic!("Unsupported selector type {}.", settings.name);
            }
        }
    }
}

impl LinkSelect<LinkProperties> for LinkSelector {
    fn select_link(
        &self,
        links: Vec<Link<LinkProperties>>,
        agent_stats: &Vec<&CommStats>,
    ) -> Vec<Link<LinkProperties>> {
        if links.len() == 1 {
            return links;
        }
        match self {
            LinkSelector::All => links,
            LinkSelector::Nearest(selector) => selector.select_link(&links),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NearestLink {}

impl NearestLink {
    fn new(_settings: &LinkSelectionSettings) -> Self {
        Self {}
    }

    pub fn select_link(&self, links: &Vec<Link<LinkProperties>>) -> Vec<Link<LinkProperties>> {
        links[0..1].to_vec()
    }
}
