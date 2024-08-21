use rand::seq::IteratorRandom;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

use crate::fl::client::AgentInfo;

#[derive(Clone, Deserialize)]
pub(crate) struct ClientSelectionSettings {
    pub(crate) c: f64,
    pub(crate) sample_size: f64,
}

#[derive(Debug, Clone)]
pub(crate) enum ClientSelector {
    Random(RandomClients),
}

impl ClientSelector {
    pub(crate) fn do_selection(&mut self) {
        match self {
            ClientSelector::Random(selector) => selector.select_clients(),
        }
    }

    pub(crate) fn selected_clients(&self) -> &Vec<AgentId> {
        match self {
            ClientSelector::Random(selector) => &selector.selected_clients,
        }
    }

    pub(crate) fn register_client(&mut self, client_info: &AgentInfo) {
        match self {
            ClientSelector::Random(selector) => selector.register_client(client_info),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RandomClients {
    pub(crate) c: f64,
    pub(crate) sample_size: f64,
    all_clients: HashMap<AgentId, AgentInfo>,
    pub(crate) selected_clients: Vec<AgentId>,
}

impl RandomClients {
    fn new(settings: &ClientSelectionSettings) -> Self {
        Self {
            c: settings.c,
            sample_size: settings.sample_size,
            all_clients: HashMap::new(),
            selected_clients: Vec::new(),
        }
    }
}

impl RandomClients {
    fn register_client(&mut self, client_info: &AgentInfo) {
        self.all_clients
            .insert(client_info.id, client_info.to_owned());
    }

    fn select_clients(&mut self) {
        let mut rng = rand::thread_rng();
        if self.all_clients.len() == 0 {
            panic!("No client registered, cannot select clients");
        }
        let client_count =
            (self.all_clients.len() as f64 * self.c * self.sample_size).round() as usize;
        self.selected_clients = self
            .all_clients
            .clone()
            .iter()
            .choose_multiple(&mut rng, client_count)
            .into_iter()
            .map(|x| x.0.clone())
            .collect();
    }
}
