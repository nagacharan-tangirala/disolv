use std::fmt::{Debug, Display};

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, MobilityInfo,
    Movable, Orderable,
};
use disolv_core::bucket::TimeMS;

use crate::bucket::MyBucket;

#[derive(Copy, Clone, Default, Debug)]
pub struct Mobility {
    x: f32,
    y: f32,
    velocity: f32,
}

impl Mobility {
    fn new(x: f32, y: f32, velocity: f32) -> Mobility {
        Mobility { x, y, velocity }
    }
}

impl MobilityInfo for Mobility {}

#[derive(Default, Clone, Copy, Debug)]
pub struct DeviceStats {
    pub id: AgentId,
    pub device_type: AgentKind,
    pub device_class: AgentClass,
    pub order: AgentOrder,
    pub size: f32,
}

impl DeviceStats {
    pub fn new(
        id: AgentId,
        device_type: AgentKind,
        device_class: AgentClass,
        order: AgentOrder,
    ) -> Self {
        Self {
            id,
            device_type,
            device_class,
            order,
            size: 0.0,
        }
    }
}

impl AgentProperties for DeviceStats {
    fn id(&self) -> AgentId {
        self.id
    }

    fn kind(&self) -> &AgentKind {
        &self.device_type
    }

    fn class(&self) -> &AgentClass {
        &self.device_class
    }
}

#[derive(Default, Clone, Debug)]
pub struct TDevice {
    pub device_info: DeviceStats,
    pub step: TimeMS,
}

impl Activatable<MyBucket> for TDevice {
    fn activate(&mut self, my_bucket: &mut MyBucket) {}

    fn deactivate(&mut self) {}

    fn is_deactivated(&self) -> bool {
        false
    }

    fn has_activation(&self) -> bool {
        false
    }

    fn time_of_activation(&mut self) -> TimeMS {
        TimeMS::from(0)
    }
}

impl Movable<MyBucket> for TDevice {
    type M = Mobility;

    fn mobility(&self) -> &Self::M {
        todo!()
    }

    fn set_mobility(&mut self, bucket: &mut MyBucket) {
        todo!()
    }
}

impl Orderable for TDevice {
    fn order(&self) -> AgentOrder {
        self.device_info.order
    }
}

impl Agent<MyBucket> for TDevice {
    fn id(&self) -> AgentId {
        self.device_info.id
    }

    fn stage_one(&mut self, _bucket: &mut MyBucket) {
        self.device_info.size = 10000.0;
    }

    fn stage_two_reverse(&mut self, _bucket: &mut MyBucket) {
        self.device_info.size = 0.0;
    }
}

impl TDevice {
    pub fn make_device(id: AgentId, device_type: AgentKind, order: u32) -> Self {
        let device_class = AgentClass::Vehicle5G;
        let device_info = DeviceStats::new(id, device_type, device_class, AgentOrder::from(order));
        Self {
            device_info,
            step: TimeMS::default(),
        }
    }
}
