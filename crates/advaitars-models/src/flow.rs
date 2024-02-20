use advaitars_core::entity::NodeClass;
use advaitars_core::message::DPayload;
use advaitars_core::radio::{IncomingStats, OutgoingStats};
use advaitars_engine::hashbrown::HashMap;
use advaitars_engine::node::NodeId;

#[derive(Clone, Debug, Default)]
pub struct FlowRegister {
    pub out_stats: OutgoingStats,
    pub out_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    pub in_stats: IncomingStats,
    pub in_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
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
            .entry(payload.node_state.node_info.node_class)
            .or_default()
            .push(payload.node_state.node_info.id);
    }

    pub fn register_incoming(&mut self, payloads: &Vec<DPayload>) {
        payloads.iter().for_each(|payload| {
            self.in_stats.update(&payload.metadata);
            self.in_link_nodes
                .entry(payload.node_state.node_info.node_class)
                .or_default()
                .push(payload.node_state.node_info.id);
        });
    }
}
