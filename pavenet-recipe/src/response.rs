use crate::node_info::id::NodeId;
use crate::node_info::kind::NodeType;
use crate::payload::DataType;
use pavenet_core::download::{RequestCreek, Response, TransferStats};
use serde_derive::Deserialize;

pub type TResponse = Response<NodeId, DataType, DataSource, TransferMetrics>;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub target_type: NodeType,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

impl RequestCreek<DataType> for DataSource {}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct TransferMetrics {
    pub latency: f32,
    pub status: bool,
}

impl TransferStats for TransferMetrics {}
