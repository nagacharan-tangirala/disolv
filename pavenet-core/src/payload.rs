use crate::entity::class::NodeClass;
use crate::entity::NodeInfo;
use crate::link::DLink;
use crate::mobility::MapState;
use crate::rules::{DTxRule, TxAction};
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::payload::{GPayload, PayloadContent, PayloadMetadata, PayloadStatus};
use pavenet_engine::response::Queryable;
use serde::Deserialize;
use typed_builder::TypedBuilder;

pub type DPayload = GPayload<NodeContent, PayloadInfo, DataType>;

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

impl PayloadStatus for TransferStatus {
    fn as_u8(&self) -> u8 {
        match self {
            TransferStatus::Ok => 1,
            TransferStatus::Fail => 0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NodeContent {
    pub node_info: NodeInfo,
    pub map_state: MapState,
}

impl PayloadContent for NodeContent {}

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct PayloadTxInfo {
    pub selected_link: DLink,
    pub tx_order: Option<u32>,
    pub status: TransferStatus,
    pub next_hop: NodeClass,
    pub final_target: NodeClass,
    pub fwd_actions: HashMap<DataType, TxAction>,
}

#[derive(Clone, Debug, Default)]
pub struct PayloadInfo {
    pub total_size: f32,
    pub total_count: u32,
    pub size_by_type: HashMap<DataType, f32>,
    pub count_by_type: HashMap<DataType, u32>,
    pub tx_info: PayloadTxInfo,
}

impl PayloadInfo {
    pub fn apply_rule(&mut self, tx_rule: &DTxRule) {
        match tx_rule.action {
            TxAction::Consume => self.consume(&tx_rule.query_type),
            TxAction::ForwardToTier(node_tier) => {
                self.tx_info
                    .fwd_actions
                    .insert(tx_rule.query_type, TxAction::ForwardToTier(node_tier));
                self.tx_info.next_hop = node_tier;
            }
            TxAction::ForwardToKind(node_type) => {
                self.tx_info
                    .fwd_actions
                    .insert(tx_rule.query_type, TxAction::ForwardToKind(node_type));
            }
        };
    }

    fn consume(&mut self, data_type: &DataType) {
        self.total_size -= self.size_by_type.get(data_type).unwrap();
        self.total_count -= self.count_by_type.get(data_type).unwrap();
        self.size_by_type.remove(data_type);
        self.count_by_type.remove(data_type);
    }
}

impl PayloadMetadata<DataType> for PayloadInfo {}
