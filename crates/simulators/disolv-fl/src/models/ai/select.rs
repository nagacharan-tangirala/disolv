use std::ops::Add;

use hashbrown::HashMap;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::model::{Model, ModelSettings};

use crate::fl::device::DeviceInfo;
use crate::models::ai::filter::{ClientFilter, ClientFilterSettings};

#[derive(Clone, Deserialize)]
pub(crate) struct ClientSelectionSettings {
    pub(crate) filter_1_settings: Option<ClientFilterSettings>,
    pub(crate) filter_2_settings: Option<ClientFilterSettings>,
}

impl ModelSettings for ClientSelectionSettings {}

#[derive(Debug, Clone)]
pub(crate) struct ClientSelector {
    all_clients: HashMap<AgentId, DeviceInfo>,
    pub(crate) selected_clients: Vec<AgentId>,
    pub(crate) filter_1: ClientFilter,
    pub(crate) filter_2: ClientFilter,
}

impl Model for ClientSelector {
    type Settings = ClientSelectionSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        let filter_1 = match &settings.filter_1_settings {
            Some(val) => ClientFilter::with_settings(val),
            None => ClientFilter::NoFilter,
        };
        let filter_2 = match &settings.filter_2_settings {
            Some(val) => ClientFilter::with_settings(val),
            None => ClientFilter::NoFilter,
        };
        Self {
            all_clients: HashMap::new(),
            selected_clients: Vec::new(),
            filter_1,
            filter_2,
        }
    }
}

impl ClientSelector {
    pub(crate) fn do_selection(&mut self) {
        let filtered_clients = self.filter_1.filter_clients(&self.all_clients);
        let feasible_clients = self.filter_2.filter_clients(&filtered_clients);

        self.selected_clients = feasible_clients.clone().into_keys().collect();

        self.filter_1.update_used_clients(&self.selected_clients);
        self.filter_2.update_used_clients(&self.selected_clients);
    }

    pub(crate) fn selected_clients(&self) -> &Vec<AgentId> {
        &self.selected_clients
    }

    pub(crate) fn selected_as_string(&self) -> String {
        let mut clients = String::new();
        clients.push('|');
        self.selected_clients.iter().for_each(|client| {
            clients.push_str(client.to_string().as_str());
            clients.push('|');
        });
        return clients;
    }

    pub(crate) fn register_client(&mut self, client_info: &DeviceInfo) {
        self.all_clients
            .insert(client_info.id, client_info.to_owned());
    }

    pub(crate) fn clear_states(&mut self) {
        self.all_clients.clear()
    }

    pub(crate) fn registered_count(&self) -> usize {
        self.all_clients.len()
    }

    pub(crate) fn has_clients(&self) -> bool {
        !self.all_clients.is_empty()
    }
}
