use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

use crate::fl::client::AgentInfo;

#[derive(Debug, Clone)]
pub(crate) enum ClientSelector {
    Random(RandomClients),
}

pub(crate) trait SelectClients {
    fn select_clients(&self, agent_infos: &HashMap<AgentId, AgentInfo>);
}

#[derive(Debug, Clone)]
pub(crate) struct RandomClients {
    pub(crate) c: f64,
}

impl RandomClients {
    fn new(c: f64) -> Self {
        Self { c: c }
    }
}

impl SelectClients for RandomClients {
    fn select_clients(&self, agent_infos: &HashMap<AgentId, AgentInfo>) {
        todo!()
    }
}
