use crate::model::{NodeModel, TomlReadable};
use crate::node_info::payload::Payload;
use pavenet_core::types::NodeId;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

#[derive(Clone, Debug, Copy)]
pub enum ResponderType {
    Stats(StatsResponder),
}

impl ResponderType {
    pub fn to_input(&self) -> ResponderSettings {
        match self {
            ResponderType::Stats(responder) => responder.to_input(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct StatsResponder;

impl NodeModel for StatsResponder {
    type Input = ResponderSettings;
    fn to_input(&self) -> ResponderSettings {
        let name: String = "stats".to_string();
        ResponderSettings { name }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DownlinkPayload {
    pub(crate) id: NodeId,
    pub(crate) latency_factor: u32,
}

impl StatsResponder {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
