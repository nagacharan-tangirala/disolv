use pavenet_engine::entity::{Kind, Tier};
use pavenet_engine::node::NodeId;
use serde::Deserialize;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: NodeClass,
}

#[derive(Deserialize, Default, Clone, Copy, Debug)]
pub enum NodeClass {
    #[default]
    None,
    Vehicle5G,
    RSU5G,
    BaseStation5G,
    Controller,
}

#[derive(Deserialize, Debug, Copy, Default, Clone, PartialEq, Eq, Hash)]
pub struct NodeOrder(pub u32);

impl Tier for NodeOrder {
    fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

#[derive(Deserialize, Debug, Hash, Copy, Default, Clone, PartialEq, Eq)]
pub enum NodeType {
    #[default]
    Vehicle = 0,
    RSU,
    BaseStation,
    Controller,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Vehicle => write!(f, "Vehicle"),
            NodeType::RSU => write!(f, "RSU"),
            NodeType::BaseStation => write!(f, "BaseStation"),
            NodeType::Controller => write!(f, "Controller"),
        }
    }
}

impl Kind for NodeType {}
