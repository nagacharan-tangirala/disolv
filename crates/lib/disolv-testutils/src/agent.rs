use std::fmt::{Debug, Display, Formatter};

use disolv_core::agent::{
    Activatable, Agent, AgentId, AgentKind, AgentOrder, AgentStats, MobilityInfo, Movable,
    Orderable,
};
use disolv_core::bucket::TimeMS;
use disolv_core::core::Core;

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

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceType {
    #[default]
    TypeA,
    TypeB,
}

impl AgentKind for DeviceType {}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::TypeA => write!(f, "TypeA"),
            DeviceType::TypeB => write!(f, "TypeB"),
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct DeviceStats {
    pub size: f32,
}

impl AgentStats for DeviceStats {}

#[derive(Default, Clone, Debug)]
pub struct TDevice {
    pub id: AgentId,
    pub device_type: DeviceType,
    pub order: AgentOrder,
    pub(crate) stats: DeviceStats,
    pub(crate) step: TimeMS,
}

impl Activatable for TDevice {
    fn activate(&mut self) {}

    fn deactivate(&mut self) {}

    fn is_deactivated(&self) -> bool {
        false
    }

    fn time_to_activation(&mut self) -> TimeMS {
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
        self.order
    }
}

impl Agent<MyBucket> for TDevice {
    type AS = DeviceStats;

    fn id(&self) -> AgentId {
        self.id
    }

    fn stats(&self) -> Self::AS {
        self.stats
    }

    fn stage_one(&mut self, _core: &mut Core<DeviceStats, MyBucket>) {
        self.stats.size = 10000.0;
    }

    fn stage_two_reverse(&mut self, _core: &mut Core<DeviceStats, MyBucket>) {
        self.stats.size = 0.0;
    }
}

impl TDevice {
    pub fn make_device(id: AgentId, device_type: DeviceType, order: i32) -> Self {
        Self {
            id,
            device_type,
            order: AgentOrder::from(order as u32),
            stats: Default::default(),
            step: TimeMS::default(),
        }
    }
}
