use crate::entity::kind::NodeType;
use crate::payload::{DataType, TransferStatus};
use pavenet_engine::response::{GResponse, ResponseContent, ResponseMetadata};
use serde::Deserialize;

pub type DResponse = GResponse<DataSource, TransferMetrics, DataType>;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub first_hop: NodeType,
    pub final_target: NodeType,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

impl ResponseContent<DataType> for DataSource {}

#[derive(Debug, Clone, Default, Copy)]
pub struct TransferMetrics {
    pub latency: f32,
    pub transfer_status: TransferStatus,
}

impl ResponseMetadata for TransferMetrics {}

impl TransferMetrics {
    pub fn new(latency: f32) -> Self {
        Self {
            latency,
            ..Default::default()
        }
    }
}
