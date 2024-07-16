use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

use crate::device::types::DeviceClass;
use crate::net::message::V2XPayload;
use crate::net::radio::{IncomingStats, OutgoingStats};

#[derive(Clone, Debug, Default)]
pub struct FlowRegister {
    pub out_stats: OutgoingStats,
    pub out_link_agents: HashMap<DeviceClass, Vec<AgentId>>,
    pub in_stats: IncomingStats,
    pub in_link_agents: HashMap<DeviceClass, Vec<AgentId>>,
}

impl FlowRegister {
    pub fn reset(&mut self) {
        self.out_stats.reset();
        self.in_link_agents.clear();
        self.out_stats.reset();
        self.out_link_agents.clear();
    }

    pub fn register_outgoing_attempt(&mut self, payload: &V2XPayload) {
        self.out_stats.add_attempted(&payload.metadata);
    }

    pub fn register_outgoing_feasible(&mut self, payload: &V2XPayload) {
        self.out_stats.add_feasible(&payload.metadata);
        self.out_link_agents
            .entry(payload.agent_state.device_info.device_class)
            .or_default()
            .push(payload.agent_state.device_info.id);
    }

    pub fn register_incoming(&mut self, payloads: &Vec<V2XPayload>) {
        payloads.iter().for_each(|payload| {
            self.in_stats.update(&payload.metadata);
            self.in_link_agents
                .entry(payload.agent_state.device_info.device_class)
                .or_default()
                .push(payload.agent_state.device_info.id);
        });
    }
}
