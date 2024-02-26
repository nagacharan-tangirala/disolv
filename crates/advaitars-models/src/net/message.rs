use crate::device::mobility::MapState;
use crate::device::types::{DeviceClass, DeviceInfo};
use crate::net::metrics::{Bandwidth, Bytes, Latency};
use crate::net::radio::{Action, ActionType, DLink};
use advaitars_core::agent::AgentId;
use advaitars_core::bucket::TimeMS;
use advaitars_core::message::{DataUnit, GPayload, Metadata, NodeState, PayloadStatus};
use advaitars_core::message::{GResponse, Queryable, Reply, TxReport};
use advaitars_core::uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Default, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    #[default]
    CAM,
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::CAM => write!(f, "CAM"),
            DataType::Image => write!(f, "Image"),
            DataType::Video => write!(f, "Video"),
            DataType::Lidar2D => write!(f, "Lidar2D"),
            DataType::Lidar3D => write!(f, "Lidar3D"),
            DataType::Radar => write!(f, "Radar"),
        }
    }
}

impl Queryable for DataType {}

#[derive(Copy, Clone, Debug, Default)]
pub struct DeviceContent {
    pub device_info: DeviceInfo,
    pub map_state: MapState,
}

impl NodeState for DeviceContent {}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct DataBlob {
    pub data_type: DataType,
    pub data_size: Bytes,
    pub action: Action,
}

impl DataUnit for DataBlob {}

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct PayloadInfo {
    pub id: Uuid,
    pub total_size: Bytes,
    pub total_count: u32,
    pub data_blobs: Vec<DataBlob>,
    pub selected_link: DLink,
}

impl PayloadInfo {
    pub fn consume(&mut self) {
        self.data_blobs.iter_mut().for_each(|blob| {
            if blob.action.action_type == ActionType::Consume {
                self.total_size -= blob.data_size;
                self.total_count -= 1;
            }
        });
        self.data_blobs
            .retain(|blob| blob.action.action_type != ActionType::Consume);
    }
}

impl Metadata for PayloadInfo {}

pub type DPayload = GPayload<PayloadInfo, DeviceContent>;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub agent_class: DeviceClass,
    pub data_size: Bytes,
    pub source_step: TimeMS,
}

impl Reply for DataSource {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum TxStatus {
    Composed,
    Ok,
    #[default]
    Fail,
}

impl PayloadStatus for TxStatus {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum TxFailReason {
    #[default]
    None,
    LatencyLimit,
    NoBandwidth,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TxMetrics {
    pub from_node: AgentId,
    pub tx_order: u32,
    pub tx_status: TxStatus,
    pub payload_size: Bytes,
    pub tx_fail_reason: TxFailReason,
    pub link_found_at: TimeMS,
    pub latency: Latency,
    pub bandwidth: Bandwidth,
}

impl TxMetrics {
    pub fn new(payload: &DPayload, tx_order: u32) -> Self {
        Self {
            from_node: payload.node_state.device_info.id,
            payload_size: payload.metadata.total_size,
            tx_order,
            ..Default::default()
        }
    }
}

impl TxReport for TxMetrics {}

pub type DResponse = GResponse<DataSource, TxMetrics>;
