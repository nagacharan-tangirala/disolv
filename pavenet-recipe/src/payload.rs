use crate::mobility::MapState;
use crate::node_info::id::NodeId;
use crate::node_info::NodeInfo;
use hashbrown::HashMap;
use pavenet_core::download::Queryable;
use pavenet_core::upload::{DataCreek, Payload, PayloadData, PayloadMetadata};
use serde_derive::Deserialize;

pub type TPayloadData = PayloadData<PayloadContent, NodeId, DataType>;
pub type TPayload = Payload<PayloadContent, NodeId, PayloadStats, DataType>;

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

impl Queryable for DataType {}

#[derive(Copy, Clone, Debug, Default)]
pub struct PayloadContent {
    pub node_info: NodeInfo,
    pub map_state: MapState,
}

impl DataCreek<DataType> for PayloadContent {}

#[derive(Clone, Debug, Default)]
pub struct PayloadStats {
    pub total_size: f32,
    pub total_count: u32,
    pub size_by_type: HashMap<DataType, f32>,
    pub count_by_type: HashMap<DataType, u32>,
}

impl PayloadMetadata<PayloadContent, DataType> for PayloadStats {}
