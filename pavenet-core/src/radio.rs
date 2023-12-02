use crate::entity::{NodeClass, NodeType};
use crate::message::{DataType, PayloadInfo};
use crate::metrics::Latency;
use pavenet_engine::node::NodeId;
use pavenet_engine::radio::{Action, GLink, IncomingStats, LinkFeatures, OutgoingStats};
use serde::Deserialize;
use std::fmt::Display;
use typed_builder::TypedBuilder;

#[derive(Debug, Copy, Clone, Default)]
pub struct LinkProperties {
    pub distance: Option<f32>,
    pub load_factor: Option<f32>,
}

impl LinkFeatures for LinkProperties {}

pub type DLink = GLink<LinkProperties>;

#[derive(Deserialize, Clone, Debug, Copy, Default)]
pub enum ActionType {
    #[default]
    Consume,
    Forward,
}

#[derive(Clone, Default, Debug, Copy, TypedBuilder)]
pub struct ActionImpl {
    pub action_type: ActionType,
    pub to_class: Option<NodeClass>,
    pub to_node: Option<NodeId>,
    pub to_kind: Option<NodeType>,
}

impl Action for ActionImpl {}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde_with::skip_serializing_none]
pub struct DecisionSettings {
    pub source: NodeClass,
    pub data_type: DataType,
    pub action_type: ActionType,
    pub to_class: Option<NodeClass>,
    pub to_node: Option<NodeId>,
    pub to_kind: Option<NodeType>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Counts {
    pub node_count: u32,
    pub data_size: f32,
    pub data_count: u32,
}

impl Counts {
    pub fn reset(&mut self) {
        self.node_count = 0;
        self.data_size = 0.0;
        self.data_count = 0;
    }
}

impl Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(node_count: {}, data_size: {}, data_count: {})",
            self.node_count, self.data_size, self.data_count
        )
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct InDataStats {
    pub attempted: Counts,
    pub feasible: Counts,
    pub avg_latency: Latency,
}

impl Display for InDataStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(attempted: {}, feasible: {}, avg_latency: {})",
            self.attempted, self.feasible, self.avg_latency
        )
    }
}

impl InDataStats {
    pub fn new() -> Self {
        InDataStats::default()
    }

    pub fn reset(&mut self) {
        self.attempted.reset();
        self.feasible.reset();
        self.avg_latency = Latency::default();
    }

    pub fn get_success_rate(&self) -> f32 {
        self.feasible.node_count as f32 / self.attempted.node_count as f32
    }
}

impl IncomingStats<PayloadInfo> for InDataStats {
    fn add_attempted(&mut self, metadata: &PayloadInfo) {
        self.attempted.node_count += 1;
        self.attempted.data_size += metadata.total_size;
        self.attempted.data_count += metadata.total_count;
    }

    fn add_feasible(&mut self, metadata: &PayloadInfo) {
        self.feasible.node_count += 1;
        self.feasible.data_size += metadata.total_size;
        self.feasible.data_count += metadata.total_count;
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct OutDataStats {
    pub out_counts: Counts,
}

impl OutDataStats {
    pub fn new() -> Self {
        OutDataStats::default()
    }

    pub fn reset(&mut self) {
        self.out_counts.reset();
    }
}

impl OutgoingStats<PayloadInfo> for OutDataStats {
    fn update(&mut self, metadata: &PayloadInfo) {
        self.out_counts.node_count += 1;
        self.out_counts.data_size += metadata.total_size;
        self.out_counts.data_count += metadata.total_count;
    }
}
