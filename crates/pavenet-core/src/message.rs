use crate::entity::{NodeClass, NodeInfo};
use crate::metrics::Latency;
use crate::mobility::MapState;
use crate::radio::{ActionImpl, ActionType, DLink};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::message::{DataUnit, GPayload, Metadata, NodeState, PayloadStatus};
use pavenet_engine::message::{GResponse, Queryable, Reply, RxReport};
use pavenet_engine::node::NodeId;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use typed_builder::TypedBuilder;
use uuid::Uuid;

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
pub struct NodeContent {
    pub node_info: NodeInfo,
    pub map_state: MapState,
}

impl NodeState for NodeContent {}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct DataBlob {
    pub data_type: DataType,
    pub data_size: f32,
    pub action: ActionImpl,
}

impl DataUnit for DataBlob {
    type Action = ActionImpl;
    fn size(&self) -> f32 {
        self.data_size
    }

    fn action(&self) -> Self::Action {
        self.action
    }

    fn set_action(&mut self, action: Self::Action) {
        self.action = action;
    }
}

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct PayloadInfo {
    pub id: Uuid,
    pub total_size: f32,
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

pub type DPayload = GPayload<PayloadInfo, NodeContent>;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataSource {
    pub data_type: DataType,
    pub node_class: NodeClass,
    pub data_size: f32,
    pub source_step: TimeMS,
}

impl Reply for DataSource {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum RxStatus {
    Composed,
    Ok,
    #[default]
    Fail,
}

impl PayloadStatus for RxStatus {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize, Default)]
pub enum RxFailReason {
    #[default]
    None,
    LatencyLimit,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct RxMetrics {
    pub from_node: NodeId,
    pub rx_order: u32,
    pub rx_status: RxStatus,
    pub payload_size: f32,
    pub rx_fail_reason: RxFailReason,
    pub link_found_at: TimeMS,
    pub latency: Latency,
}

impl RxMetrics {
    pub fn new(payload: &DPayload, rx_order: u32) -> Self {
        Self {
            from_node: payload.node_state.node_info.id,
            payload_size: payload.metadata.total_size,
            rx_order,
            ..Default::default()
        }
    }
}

impl RxReport for RxMetrics {}

pub type DResponse = GResponse<DataSource, RxMetrics>;
