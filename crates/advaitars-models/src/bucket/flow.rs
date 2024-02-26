use crate::device::types::DeviceClass;
use crate::net::message::DPayload;
use crate::net::radio::{IncomingStats, OutgoingStats};
use advaitars_core::agent::AgentId;
use advaitars_core::hashbrown::HashMap;

#[derive(Clone, Debug, Default)]
pub struct FlowRegister {
    pub out_stats: OutgoingStats,
    pub out_link_nodes: HashMap<DeviceClass, Vec<AgentId>>,
    pub in_stats: IncomingStats,
    pub in_link_nodes: HashMap<DeviceClass, Vec<AgentId>>,
}

impl FlowRegister {
    pub fn reset(&mut self) {
        self.out_stats.reset();
        self.in_link_nodes.clear();
        self.out_stats.reset();
        self.out_link_nodes.clear();
    }

    pub fn register_outgoing_attempt(&mut self, payload: &DPayload) {
        self.out_stats.add_attempted(&payload.metadata);
    }

    pub fn register_outgoing_feasible(&mut self, payload: &DPayload) {
        self.out_stats.add_feasible(&payload.metadata);
        self.out_link_nodes
            .entry(payload.node_state.device_info.device_class)
            .or_default()
            .push(payload.node_state.device_info.id);
    }

    pub fn register_incoming(&mut self, payloads: &Vec<DPayload>) {
        payloads.iter().for_each(|payload| {
            self.in_stats.update(&payload.metadata);
            self.in_link_nodes
                .entry(payload.node_state.device_info.device_class)
                .or_default()
                .push(payload.node_state.device_info.id);
        });
    }
}
