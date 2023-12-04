use crate::entity::{NodeClass, NodeInfo};
use crate::metrics::Latency;
use crate::mobility::MapState;
use crate::radio::{ActionImpl, ActionType, DLink};
use pavenet_engine::bucket::TimeS;
use pavenet_engine::message::{DataUnit, GPayload, Metadata, NodeState, PayloadStatus};
use pavenet_engine::message::{GResponse, Queryable, Reply, TxStatus};
use serde::Deserialize;
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

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct DataBlob {
    pub uuid: Uuid,
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
pub struct TxInfo {
    pub selected_link: DLink,
    pub link_found_at: TimeS,
    pub tx_order: Option<u32>,
    pub status: TransferStatus,
}

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct PayloadInfo {
    pub total_size: f32,
    pub total_count: u32,
    pub data_blobs: Vec<DataBlob>,
    pub routing_info: TxInfo,
}

impl PayloadInfo {
    pub fn consume(&mut self) {
        self.data_blobs.iter_mut().for_each(|blob| {
            self.total_size -= blob.data_size;
            self.total_count -= 1;
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
    pub source_step: TimeS,
}

impl Reply for DataSource {}

#[derive(Clone, Eq, PartialEq, Copy, Debug, Default)]
pub enum TransferStatus {
    Ok,
    #[default]
    Fail,
}

impl PayloadStatus for TransferStatus {
    fn as_u8(&self) -> u8 {
        match self {
            TransferStatus::Ok => 1,
            TransferStatus::Fail => 0,
        }
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TransferMetrics {
    pub latency: Latency,
    pub transfer_status: TransferStatus,
}

impl TxStatus for TransferMetrics {}

impl TransferMetrics {
    pub fn new(latency: Latency) -> Self {
        Self {
            latency,
            ..Default::default()
        }
    }
}

pub type DResponse = GResponse<DataSource, TransferMetrics>;
