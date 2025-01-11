use std::fmt::Display;

use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentId, AgentProperties};
use disolv_core::bucket::TimeMS;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType, TxReport};
use disolv_core::metrics::Bytes;
use disolv_core::radio::{Action, Link};
use disolv_models::net::metrics::Bandwidth;
use disolv_models::net::radio::LinkProperties;

use crate::fl::device::DeviceInfo;

/// These are type of messages exchanged in a typical training context.
#[derive(Deserialize, Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Message {
    #[default]
    Sensor,
    FlMessage,
}

impl QueryType for Message {}

/// These are the format of the message exchanged.
#[derive(Deserialize, Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MessageType {
    #[default]
    KiloByte, // Could be state requests, selection acknowledgement etc.
    SensorData,
    U32Weights,
    F32Weights,
    F64Weights,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::KiloByte => write!(f, "KiloByte"),
            MessageType::SensorData => write!(f, "SensorData"),
            MessageType::U32Weights => write!(f, "U32Weights"),
            MessageType::F32Weights => write!(f, "F32Weights"),
            MessageType::F64Weights => write!(f, "F64Weights"),
        }
    }
}

impl ContentType for MessageType {}

#[derive(Deserialize, Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum FlAction {
    #[default]
    None,
    StateInfo,
    GlobalModel,
    GlobalModelReceived,
    LocalModel,
    InitiateTraining,
    CompleteTraining,
    TrainingFailed,
    Training,
}

impl Display for FlAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlAction::None => write!(f, "None"),
            FlAction::StateInfo => write!(f, "StateInfo"),
            FlAction::GlobalModel => write!(f, "GlobalModel"),
            FlAction::GlobalModelReceived => write!(f, "GlobalModelReceived"),
            FlAction::LocalModel => write!(f, "LocalModel"),
            FlAction::InitiateTraining => write!(f, "InitiateTraining"),
            FlAction::CompleteTraining => write!(f, "CompleteTraining"),
            FlAction::Training => write!(f, "Training"),
            FlAction::TrainingFailed => write!(f, "TrainingFailed"),
        }
    }
}

/// A single unit of a message, a collection of these messages can be sent by a single device.
/// Hence, this is called a unit. It contains the instructions as to what to do with the message
/// and some metadata.
#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct MessageUnit {
    pub action: Action,
    pub fl_action: FlAction,
    pub message_type: MessageType,
    pub message_size: Bytes,
    pub device_info: DeviceInfo,
}

impl DataUnit<MessageType> for MessageUnit {
    fn action(&self) -> &Action {
        &self.action
    }

    fn content_type(&self) -> &MessageType {
        &self.message_type
    }

    fn size(&self) -> Bytes {
        self.message_size
    }

    fn update_action(&mut self, new_action: &Action) {
        self.action.action_type = new_action.action_type;
        self.action.to_agent = new_action.to_agent;
        self.action.to_class = new_action.to_class;
        self.action.to_kind = new_action.to_kind;
        self.action.to_broadcast = new_action.to_broadcast.to_owned();
    }
}

/// A link with properties.
pub type FlLink = Link<LinkProperties>;

/// This represents the metadata of the entire payload.
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct FlPayloadInfo {
    pub total_size: Bytes,
    pub total_count: u32,
    pub selected_link: FlLink,
}

impl Metadata for FlPayloadInfo {
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

pub type FlPayload = Payload<MessageType, MessageUnit, FlPayloadInfo, DeviceInfo, Message>;

#[derive(Debug, Clone, Default, Copy)]
pub struct TxMetrics {
    pub from_agent: AgentId,
    pub tx_order: u32,
    pub payload_size: Bytes,
    pub link_found_at: TimeMS,
    pub bandwidth: Bandwidth,
}

impl TxMetrics {
    pub fn new(payload: &FlPayload, tx_order: u32) -> Self {
        Self {
            from_agent: payload.agent_state.id(),
            payload_size: payload.metadata.total_size,
            tx_order,
            ..Default::default()
        }
    }
}

impl TxReport for TxMetrics {}
