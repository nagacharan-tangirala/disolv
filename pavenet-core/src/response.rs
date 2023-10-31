use crate::entity::class::NodeClass;
use crate::payload::{DataType, TransferStatus};
use crate::radio::metrics::latency::Latency;
use pavenet_engine::response::{GResponse, ResponseContent, ResponseMetadata};
use serde::Deserialize;

pub type DResponse = GResponse<DataSource, TransferMetrics, DataType>;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub node_class: NodeClass,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

impl ResponseContent<DataType> for DataSource {}

#[derive(Debug, Clone, Default, Copy)]
pub struct TransferMetrics {
    pub latency: Latency,
    pub transfer_status: TransferStatus,
}

impl ResponseMetadata for TransferMetrics {}

impl TransferMetrics {
    pub fn new(latency: Latency) -> Self {
        Self {
            latency,
            ..Default::default()
        }
    }
}
