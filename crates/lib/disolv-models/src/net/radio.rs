use std::fmt::Display;

use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId, AgentKind};
use disolv_core::message::{ContentType, Metadata};
use disolv_core::metrics::Bytes;
use disolv_core::radio::{ActionType, LinkFeatures};

use crate::net::metrics::Latency;

#[derive(Debug, Copy, Clone, Default)]
pub struct LinkProperties {
    pub distance: Option<f32>,
    pub load_factor: Option<f32>,
}

impl LinkFeatures for LinkProperties {}

#[derive(Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
pub struct ActionSettings<C: ContentType> {
    pub target: AgentClass,
    pub data_type: C,
    pub action_type: ActionType,
    pub to_class: Option<AgentClass>,
    pub to_agent: Option<AgentId>,
    pub to_kind: Option<AgentKind>,
    pub to_broadcast: Option<Vec<AgentId>>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Counts {
    pub agent_count: u32,
    pub data_size: Bytes,
    pub data_count: u32,
}

impl Counts {
    pub fn reset(&mut self) {
        self.agent_count = 0;
        self.data_size = Bytes::default();
        self.data_count = 0;
    }
}

impl Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(agent_count: {}, data_size: {}, data_count: {})",
            self.agent_count, self.data_size, self.data_count
        )
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct OutgoingStats {
    pub attempted: Counts,
    pub feasible: Counts,
    pub avg_latency: Latency,
}

impl Display for OutgoingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(attempted: {}, feasible: {}, avg_latency: {})",
            self.attempted, self.feasible, self.avg_latency
        )
    }
}

impl OutgoingStats {
    pub fn reset(&mut self) {
        self.attempted.reset();
        self.feasible.reset();
        self.avg_latency = Latency::default();
    }

    pub fn get_success_rate(&self) -> f32 {
        if self.attempted.agent_count == 0 {
            return 0.0;
        }
        self.feasible.agent_count as f32 / self.attempted.agent_count as f32
    }

    pub fn add_attempted<M: Metadata>(&mut self, metadata: &M) {
        self.attempted.agent_count += 1;
        self.attempted.data_size += metadata.size();
        self.attempted.data_count += metadata.count();
    }

    pub fn add_feasible<M: Metadata>(&mut self, metadata: &M) {
        self.feasible.agent_count += 1;
        self.feasible.data_size += metadata.size();
        self.feasible.data_count += metadata.count();
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct IncomingStats {
    pub in_counts: Counts,
}

impl IncomingStats {
    pub fn reset(&mut self) {
        self.in_counts.reset();
    }

    pub fn update<M: Metadata>(&mut self, metadata: &M) {
        self.in_counts.agent_count += 1;
        self.in_counts.data_size += metadata.size();
        self.in_counts.data_count += metadata.count();
    }
}

#[derive(Copy, Clone, Debug, Default, TypedBuilder)]
pub struct CommStats {
    pub outgoing_stats: OutgoingStats,
    pub incoming_stats: IncomingStats,
}

impl CommStats {
    pub fn reset(&mut self) {
        self.incoming_stats.reset();
        self.outgoing_stats.reset();
    }
}
