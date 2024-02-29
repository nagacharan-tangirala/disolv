use crate::net::message::DeviceContent;
use crate::net::radio::{IncomingStats, OutgoingStats};
use disolv_core::agent::{AgentClass, AgentId, AgentKind, AgentOrder, AgentStats};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct DeviceInfo {
    pub id: AgentId,
    pub device_type: DeviceType,
    pub device_class: DeviceClass,
    pub agent_order: AgentOrder,
}

#[derive(Deserialize, Default, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DeviceClass {
    #[default]
    None,
    Vehicle5G,
    RSU5G,
    BaseStation5G,
    Controller,
}

impl Display for DeviceClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceClass::None => write!(f, "None"),
            DeviceClass::Vehicle5G => write!(f, "Vehicle5G"),
            DeviceClass::RSU5G => write!(f, "RSU5G"),
            DeviceClass::BaseStation5G => write!(f, "BaseStation5G"),
            DeviceClass::Controller => write!(f, "Controller"),
        }
    }
}

impl AgentClass for DeviceClass {}

#[derive(Deserialize, Debug, Hash, Copy, Default, Clone, PartialEq, Eq)]
pub enum DeviceType {
    #[default]
    Vehicle = 0,
    RSU,
    BaseStation,
    Controller,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Vehicle => write!(f, "Vehicle"),
            DeviceType::RSU => write!(f, "RSU"),
            DeviceType::BaseStation => write!(f, "BaseStation"),
            DeviceType::Controller => write!(f, "Controller"),
        }
    }
}

impl AgentKind for DeviceType {}

#[derive(Copy, Clone, Debug, Default, TypedBuilder)]
pub struct DeviceStats {
    pub outgoing_stats: OutgoingStats,
    pub incoming_stats: IncomingStats,
    pub device_content: DeviceContent,
}

impl AgentStats for DeviceStats {}
