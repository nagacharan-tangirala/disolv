use pavenet_engine::entity::{Class, Kind, Tier};
use pavenet_engine::node::NodeId;
use serde::Deserialize;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: NodeClass,
    pub node_order: NodeOrder,
}

#[derive(Deserialize, Default, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum NodeClass {
    #[default]
    None,
    Vehicle5G,
    RSU5G,
    BaseStation5G,
    Controller,
}

impl Display for NodeClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeClass::None => write!(f, "None"),
            NodeClass::Vehicle5G => write!(f, "Vehicle5G"),
            NodeClass::RSU5G => write!(f, "RSU5G"),
            NodeClass::BaseStation5G => write!(f, "BaseStation5G"),
            NodeClass::Controller => write!(f, "Controller"),
        }
    }
}

impl Class for NodeClass {}

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
