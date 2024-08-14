use std::fmt::Display;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentId, AgentProperties};
use disolv_core::bucket::TimeMS;
use disolv_core::message::{
    ContentType, DataUnit, Metadata, Payload, PayloadStatus, QueryType, TxReport,
};
use disolv_core::metrics::Bytes;
use disolv_core::radio::{Action, Link};
use disolv_core::uuid::Uuid;
use disolv_models::net::metrics::{Bandwidth, Latency};
use disolv_models::net::radio::LinkProperties;

use crate::v2x::device::DeviceInfo;

pub type V2XPayload = Payload<DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType>;

/// A set of messages used by devices in Federated Learning context.
#[derive(Deserialize, Default, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    #[default]
    Sensor,
}

impl QueryType for MessageType {}

/// Types of data allowed to be transferred in a V2X scenario context.
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

impl ContentType for DataType {}

/// Define a struct that contains the metadata about the sensor data.
#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct DataBlob {
    pub action: Action,
    pub data_type: DataType,
    pub data_size: Bytes,
}

impl DataUnit<DataType> for DataBlob {
    fn action(&self) -> &Action {
        &self.action
    }

    fn content_type(&self) -> &DataType {
        &self.data_type
    }

    fn size(&self) -> Bytes {
        self.data_size
    }

    fn update_action(&mut self, new_action: &Action) {
        self.action.action_type = new_action.action_type;
        self.action.to_agent = new_action.to_agent;
        self.action.to_class = new_action.to_class;
        self.action.to_kind = new_action.to_kind;
    }
}
/// Define a struct to represent a single message that will be transferred between agents in V2X.
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct PayloadInfo {
    pub id: Uuid,
    pub total_size: Bytes,
    pub total_count: u32,
    pub selected_link: Link<LinkProperties>,
}

impl Metadata for PayloadInfo {
    fn size(&self) -> Bytes {
        self.total_size
    }

    fn count(&self) -> u32 {
        self.total_count
    }

    fn set_size(&mut self, bytes: Bytes) {
        self.total_size = bytes;
    }

    fn set_count(&mut self, count: u32) {
        self.total_count = count;
    }
}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum TxStatus {
    Ok,
    #[default]
    Fail,
}

impl TxStatus {
    pub fn as_int(&self) -> u32 {
        match self {
            TxStatus::Ok => 0,
            TxStatus::Fail => 1,
        }
    }
}

impl PayloadStatus for TxStatus {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum TxFailReason {
    #[default]
    None = 0,
    LatencyLimit,
    NoBandwidth,
}

impl TxFailReason {
    pub fn as_int(&self) -> u32 {
        match self {
            TxFailReason::None => 0,
            TxFailReason::LatencyLimit => 1,
            TxFailReason::NoBandwidth => 2,
        }
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TxMetrics {
    pub from_agent: AgentId,
    pub tx_order: u32,
    pub tx_status: TxStatus,
    pub payload_size: Bytes,
    pub tx_fail_reason: TxFailReason,
    pub link_found_at: TimeMS,
    pub latency: Latency,
    pub bandwidth: Bandwidth,
}

impl TxMetrics {
    pub fn new(payload: &V2XPayload, tx_order: u32) -> Self {
        Self {
            from_agent: payload.agent_state.id(),
            payload_size: payload.metadata.total_size,
            tx_order,
            ..Default::default()
        }
    }
}

impl TxReport for TxMetrics {}
