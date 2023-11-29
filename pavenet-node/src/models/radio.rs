use crate::models::latency::DLatencyModel;
use pavenet_core::entity::class::NodeClass;
use pavenet_engine::entity::NodeId;
use pavenet_core::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_core::radio::stats::{InDataStats, OutDataStats};
use pavenet_core::rand_pcg::Pcg64Mcg;
use pavenet_core::response::TransferMetrics;
use pavenet_core::rules::{Rules, TxAction};
use pavenet_engine::bucket::TimeS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::radio::{Channel, Feasibility, IncomingStats};
use pavenet_engine::rules::TxRuleEnforcer;
use rand::prelude::SliceRandom;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Radio {
    pub my_class: NodeClass,
    pub latency_model: DLatencyModel,
    pub step_size: TimeS,
    pub rng: Pcg64Mcg,
    #[builder(default)]
    pub in_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub out_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub in_stats: InDataStats,
    #[builder(default)]
    pub out_stats: OutDataStats,
    #[builder(default)]
    pub transfer_stats: HashMap<NodeId, TransferMetrics>,
}

impl Radio {
    fn check_feasibility(&mut self, payload: &DPayload) -> bool {
        let latency_check = match self.latency_model.check_feasibility(&payload.metadata) {
            Feasibility::Feasible(latency) => {
                self.in_stats.update_avg_latency(latency);
                self.transfer_stats
                    .insert(payload.content.node_info.id, TransferMetrics::new(latency));
                true
            }
            Feasibility::Infeasible(_) => false,
        };
        return latency_check;
    }

    pub(crate) fn transfer_stats(&mut self) -> HashMap<NodeId, TransferMetrics> {
        self.transfer_stats.clone()
    }
}

impl Channel<NodeContent, PayloadInfo, DataType, TxAction, NodeClass, Rules> for Radio {
    fn reset(&mut self) {
        self.in_stats = InDataStats::default();
        self.out_stats = OutDataStats::default();
        self.in_link_nodes.clear();
        self.out_link_nodes.clear();
        self.transfer_stats.clear();
    }

    fn can_transfer(&mut self, mut payloads: Vec<DPayload>) -> Vec<DPayload> {
        payloads.shuffle(&mut self.rng);
        let mut valid = Vec::with_capacity(payloads.len());

        for payload in payloads.into_iter() {
            self.in_stats.add_attempted(&payload.metadata);
            self.in_link_nodes
                .entry(payload.content.node_info.node_class)
                .or_insert(Vec::new())
                .push(payload.content.node_info.id);

            if !self.check_feasibility(&payload) {
                continue;
            }

            self.in_stats.add_feasible(&payload.metadata);
            valid.push(payload);
        }
        return valid;
    }

    fn apply_tx_rules(&mut self, rules: &Rules, payloads: Vec<DPayload>) -> Vec<DPayload> {
        let mut to_forward = Vec::with_capacity(payloads.len());
        for payload in payloads.into_iter() {
            let payload = rules.enforce_tx_rules(&self.my_class, payload);
            to_forward.push(payload);
        }
        return to_forward;
    }
}
