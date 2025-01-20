use std::cmp::max;

use hashbrown::{HashMap, HashSet};
use rand::seq::IteratorRandom;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::model::{Model, ModelSettings};

use crate::fl::device::DeviceInfo;

#[derive(Clone, Deserialize)]
pub struct ClientFilterSettings {
    variant: String,
    to_select: Option<f32>,
}

impl ModelSettings for ClientFilterSettings {}

#[derive(Clone, Debug)]
pub enum ClientFilter {
    NoFilter,
    Unique(UniqueFilter),
    Sample(RandomSampling),
    MaxData(MaximumDataFilter),
}

impl Model for ClientFilter {
    type Settings = ClientFilterSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "unique" => ClientFilter::Unique(UniqueFilter::with_settings(settings)),
            "random" => ClientFilter::Sample(RandomSampling::with_settings(settings)),
            "maxdata" => ClientFilter::MaxData(MaximumDataFilter::with_settings(settings)),
            _ => ClientFilter::NoFilter,
        }
    }
}

impl ClientFilter {
    pub(crate) fn filter_clients(
        &mut self,
        all_agents: &HashMap<AgentId, DeviceInfo>,
    ) -> HashMap<AgentId, DeviceInfo> {
        match self {
            ClientFilter::NoFilter => all_agents.clone(),
            ClientFilter::Unique(unique) => unique.filter_clients(all_agents),
            ClientFilter::Sample(sample) => sample.filter_clients(all_agents),
            ClientFilter::MaxData(max) => max.filter_clients(all_agents),
        }
    }

    pub(crate) fn update_used_clients(&mut self, selected_clients: &Vec<AgentId>) {
        if let ClientFilter::Unique(unique) = self {
            unique.update_used_clients(selected_clients)
        }
    }
}

#[derive(Clone, Debug)]
pub struct UniqueFilter {
    used_clients: HashSet<AgentId>,
}

impl UniqueFilter {
    fn with_settings(_settings: &ClientFilterSettings) -> Self {
        Self {
            used_clients: HashSet::new(),
        }
    }

    fn filter_clients(
        &mut self,
        all_agents: &HashMap<AgentId, DeviceInfo>,
    ) -> HashMap<AgentId, DeviceInfo> {
        let mut feasible_clients = HashMap::new();
        all_agents.clone().iter().for_each(|agent_data| {
            if !self.used_clients.contains(agent_data.0) {
                feasible_clients.insert(*agent_data.0, *agent_data.1);
            }
        });
        feasible_clients
    }

    fn update_used_clients(&mut self, clients: &Vec<AgentId>) {
        clients.iter().for_each(|client| {
            self.used_clients.insert(*client);
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RandomSampling {
    to_select: f32,
}

impl RandomSampling {
    fn with_settings(settings: &ClientFilterSettings) -> Self {
        let to_select = match settings.to_select {
            Some(val) => val,
            None => panic!("Ratio is missing from settings"),
        };
        Self { to_select }
    }

    fn filter_clients(
        &mut self,
        all_agents: &HashMap<AgentId, DeviceInfo>,
    ) -> HashMap<AgentId, DeviceInfo> {
        let mut rng = rand::thread_rng();
        let sample_count = (all_agents.len() as f32 * self.to_select).ceil() as usize;
        let filter_count = max(1, sample_count);

        all_agents
            .iter()
            .choose_multiple(&mut rng, filter_count)
            .into_iter()
            .map(|x| (*x.0, *x.1))
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MaximumDataFilter {
    to_select: f32,
}

impl MaximumDataFilter {
    fn with_settings(settings: &ClientFilterSettings) -> Self {
        let to_select = match settings.to_select {
            Some(val) => val,
            None => panic!("Ratio is missing from settings"),
        };
        Self { to_select }
    }

    fn filter_clients(
        &mut self,
        all_agents: &HashMap<AgentId, DeviceInfo>,
    ) -> HashMap<AgentId, DeviceInfo> {
        let sample_count = (all_agents.len() as f32 * self.to_select).ceil() as usize;
        let filter_count = max(1, sample_count);

        let mut sorted_agents: Vec<(&AgentId, &DeviceInfo)> = all_agents.iter().collect();
        sorted_agents.sort_by(|a, b| {
            let a1 = a.1.dynamic_info.expect("missing dynamic_info");
            let b1 = b.1.dynamic_info.expect("missing dynamic_info");
            a1.active_since.cmp(&b1.active_since)
        });

        sorted_agents
            .iter()
            .take(filter_count)
            .map(|x| (*x.0, *x.1))
            .collect()
    }
}
