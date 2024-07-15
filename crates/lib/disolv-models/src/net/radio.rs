use crate::device::types::{DeviceClass, DeviceType};
use crate::net::message::{DataType, PayloadInfo};
use crate::net::metrics::{Bytes, Latency};
use disolv_core::agent::AgentId;
use disolv_core::radio::{ActionInfo, Actionable, Actions, GLink, LinkFeatures};
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

#[derive(Deserialize, Clone, Debug, Copy, Eq, PartialEq, Default)]
pub enum ActionType {
    #[default]
    Consume,
    Forward,
}

impl Actionable for ActionType {}

#[derive(Clone, Default, Debug, Copy, TypedBuilder)]
pub struct Action {
    pub action_type: ActionType,
    pub to_class: Option<DeviceClass>,
    pub to_agent: Option<AgentId>,
    pub to_kind: Option<DeviceType>,
}

impl ActionInfo for Action {}

pub type DActions = Actions<Action, DataType>;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde_with::skip_serializing_none]
pub struct ActionSettings {
    pub target: DeviceClass,
    pub data_type: DataType,
    pub action_type: ActionType,
    pub to_class: Option<DeviceClass>,
    pub to_agent: Option<AgentId>,
    pub to_kind: Option<DeviceType>,
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

    pub fn add_attempted(&mut self, metadata: &PayloadInfo) {
        self.attempted.agent_count += 1;
        self.attempted.data_size += metadata.total_size;
        self.attempted.data_count += metadata.total_count;
    }

    pub fn add_feasible(&mut self, metadata: &PayloadInfo) {
        self.feasible.agent_count += 1;
        self.feasible.data_size += metadata.total_size;
        self.feasible.data_count += metadata.total_count;
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

    pub fn update(&mut self, metadata: &PayloadInfo) {
        self.in_counts.agent_count += 1;
        self.in_counts.data_size += metadata.total_size;
        self.in_counts.data_count += metadata.total_count;
    }
}
