use crate::utils::response::DataType;
use crate::utils::types::Nid;
use pavenet_core::payload::{DataCreek, PayloadStats};

#[derive(Clone, Copy, Default)]
pub(crate) struct SensorData {
    pub(crate) data_type: DataType,
    pub(crate) size: f32,
}

impl DataCreek<DataType> for SensorData {}

#[derive(Clone, Copy, Default)]
pub(crate) struct PayloadInfo {
    pub(crate) data_pile: SensorData,
    pub(crate) from_node: Nid,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PayloadStatData {
    pub(crate) data_size: f32,
    pub(crate) data_count: u32,
}

impl PayloadStats<SensorData, DataType> for PayloadStatData {}

#[derive(Clone, Default)]
pub(crate) struct Payload {
    pub(crate) gathered_data: Option<Vec<PayloadInfo>>,
    pub(crate) data_pile: PayloadInfo,
    pub(crate) payload_stats: PayloadStatData,
}
