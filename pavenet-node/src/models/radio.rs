use crate::models::latency::DLatencyModel;
use pavenet_core::bucket::TimeS;
use pavenet_core::entity::class::NodeClass;
use pavenet_core::entity::id::NodeId;
use pavenet_core::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_core::radio::stats::{InDataStats, OutDataStats};
use pavenet_core::rand_pcg::Pcg64Mcg;
use pavenet_core::response::TransferMetrics;
use pavenet_core::rules::{Actions, Rules};
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::radio::{Channel, Feasibility, IncomingStats};
use pavenet_engine::rules::TxRuleEnforcer;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Radio {
    pub my_class: NodeClass,
    pub latency_model: DLatencyModel,
    pub step_size: TimeS,
    pub rng: Pcg64Mcg,
    #[builder(default)]
    pub in_link_nodes: HashMap<NodeType, Vec<NodeId>>,
    #[builder(default)]
    pub out_link_nodes: HashMap<NodeType, Vec<NodeId>>,
    #[builder(default)]
    pub in_stats: InDataStats,
    #[builder(default)]
    pub out_stats: OutDataStats,
    #[builder(default)]
    pub transfer_stats: HashMap<NodeId, TransferMetrics>,
}

impl Channel<NodeContent, PayloadInfo, DataType, Actions, TimeS, Rules> for Radio {
    fn reset(&mut self) {}

    fn can_transfer(&mut self, mut payloads: Vec<DPayload>) -> Vec<DPayload> {
        payloads.shuffle(&mut self.rng);
        let mut valid = Vec::with_capacity(payloads.len());
        let mut latencies = Vec::with_capacity(payloads.len());

        for payload in payloads.into_iter() {
            self.in_stats.add_attempted(&payload.metadata);

            match self.latency_model.check_feasibility(&payload.metadata) {
                Feasibility::Feasible(latency) => {
                    latencies.push(latency);
                }
                Feasibility::Infeasible(_) => continue,
            }

            valid.push(payload);
            self.in_stats.add_feasible(&payload.metadata);
        }

        self.in_stats.update_latency(latencies);
        return valid;
    }

    fn apply_tx_rules(&mut self, rules: &Rules, mut payloads: Vec<DPayload>) -> Vec<DPayload> {
        let mut to_forward = Vec::with_capacity(payloads.len());
        for payload in payloads.into_iter() {
            let payload = rules.enforce_tx_rules(&self.my_class, payload);
            to_forward.push(payload);
        }
        return to_forward;
    }

    fn consume(&mut self, payload: &DPayload) {
        todo!()
    }
}

impl Radio {
    pub fn clear(&mut self) {
        self.in_stats = InDataStats::default();
        self.out_stats = OutDataStats::default();
        self.in_link_nodes.clear();
        self.out_link_nodes.clear();
    }
}
