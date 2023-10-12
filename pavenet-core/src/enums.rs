use serde_derive::Deserialize;
use std::fmt::Display;

#[derive(Deserialize, Debug, Clone, Default)]
pub enum MobilityType {
    #[default]
    Stationery,
    Mobile,
}

#[derive(Deserialize, Clone, Debug, Copy)]
pub enum TransferMode {
    UDT,
    BDT,
}

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
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
