use crate::link::LinkProperties;
use crate::mobility::MapState;
use crate::node_info::id::NodeId;
use crate::node_info::kind::NodeType;
use crate::node_info::NodeInfo;
use hashbrown::HashMap;
use pavenet_engine::link::Link;
use pavenet_engine::payload::{Payload, PayloadContent, PayloadMetadata, PayloadStatus};
use pavenet_engine::response::Queryable;
use serde::Deserialize;

pub type DPayload = Payload<NodeContent, PayloadInfo, DataType>;

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    CAM,
}

impl Queryable for DataType {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Default)]
pub enum TransferStatus {
    Ok,
    #[default]
    Fail,
}

impl PayloadStatus for TransferStatus {}

#[derive(Copy, Clone, Debug, Default)]
pub struct NodeContent {
    pub node_info: NodeInfo,
    pub map_state: MapState,
}

impl PayloadContent<DataType> for NodeContent {}

#[derive(Clone, Debug, Default)]
pub struct PayloadInfo {
    pub total_size: f32,
    pub total_count: u32,
    pub size_by_type: HashMap<DataType, f32>,
    pub count_by_type: HashMap<DataType, u32>,
    pub status: TransferStatus,
    pub intended_target: NodeType,
    pub tx_order: Option<u32>,
    pub selected_link: Link<LinkProperties, NodeId>,
}

impl PayloadMetadata for PayloadInfo {}
