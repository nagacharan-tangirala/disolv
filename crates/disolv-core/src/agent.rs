use super::bucket::TimeMS;
use crate::bucket::Bucket;
use crate::core::Core;
use serde::Deserialize;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use typed_builder::TypedBuilder;

/// A unique ID that is a property of all the agents in the simulation.
#[derive(Deserialize, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct AgentId(u64);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AgentId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for AgentId {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl AgentId {
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Trait that represents the kind of agent. Extend this to a custom type
/// (e.g. enum) that represents the kind of your agent. Only one instance of this type
/// is allowed. This is required to distinguish between different types of agents.
///
/// Multiple types of agents can be simulated in a single simulation. However, all agents
/// must be distinguishable by only their kind. For example, in a vehicular scenario, this trait
/// can be implemented for both vehicles and RSUs. There can also be multiple types of vehicles
/// (e.g. cars, trucks, buses, etc.). Similarly, there can be multiple types of RSUs (e.g. RSUs
/// with different transmission ranges). Each of these types can have their own struct that
/// implements the [agent] trait. However, all these types must be documented in a single enum.
/// Such enum should implement this trait.
pub trait AgentKind:
    Default + fmt::Display + Clone + Copy + PartialEq + Eq + Hash + Send + Sync
{
}

/// Trait that represents the variety within each `kind` of an agent. `kind` distinguishes
/// among the different types of devices, while `Class` will allow for varies categories within
/// each kind.
///
/// An example will be Vehicle5G, Vehicle4G being two classes of Vehicle Kind. This allows for
/// defining behaviour at both the `kind` level and `Class` level.
pub trait AgentClass:
    Default + fmt::Display + Clone + Copy + Hash + Eq + PartialEq + Send + Sync
{
}

/// Agent order indicates the order in which the behavior of the agents is simulated.
///
/// This is required to control the order of calling the uplink and downlink stages of the
/// agents. At each time step, the agents are sorted by their tier. The agents with the
/// lowest tier are called first and gradually proceeding to the agents with the highest tier.
/// This allows the agents to be simulated in a tiered fashion.
#[derive(Deserialize, Debug, Copy, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentOrder(pub u32);

impl From<u32> for AgentOrder {
    fn from(f: u32) -> Self {
        Self(f)
    }
}

impl AgentOrder {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub trait Orderable {
    fn order(&self) -> AgentOrder;
}

/// A trait that represents the mobility information of an agent. Extend this to
/// a custom type that represents the static or dynamic positional information of agents.
///
/// Multiple types of mobility information can be used in a single simulation.
/// For example, static devices need only the positional information, while mobile devices need
/// both the positional and mobility information.
pub trait MobilityInfo: Copy + Clone {}

/// A trait to get and set the mobility information of an agent. Must extend this for
/// both the static and mobile agents.
pub trait Movable<B> {
    type M: MobilityInfo;
    fn mobility(&self) -> &Self::M;
    fn set_mobility(&mut self, bucket: &mut B);
}

/// A trait that allows an agent to be scheduled for simulation.
pub trait Activatable {
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn is_deactivated(&self) -> bool;
    fn time_to_activation(&mut self) -> TimeMS;
}

/// A trait that represents an agent. Extend this to a custom device type (e.g. struct) that
/// you want to simulate. Only types with this trait can be added to a bucket and hence
/// scheduled for simulation.
///
pub trait Agent<B>: Activatable + Orderable + Movable<B> + Clone + Send + Sync
where
    B: Bucket,
{
    type AS: AgentStats;
    fn id(&self) -> AgentId;
    fn stats(&self) -> Self::AS;
    fn stage_one(&mut self, core: &mut Core<Self, B>);
    fn stage_two_reverse(&mut self, core: &mut Core<Self, B>);
    fn stage_three(&mut self, _core: &mut Core<Self, B>) {}
    fn stage_four_reverse(&mut self, _core: &mut Core<Self, B>) {}
    fn stage_five(&mut self, _core: &mut Core<Self, B>) {}
}

pub trait AgentStats: Copy + Clone + Send + Sync {}

/// A struct that represents a generic agent. This is a wrapper around the agent type that
/// implements the [Agent] trait. This is required to store the agents in the [scheduler].
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct AgentImpl<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub agent_id: AgentId,
    pub agent: A,
    #[builder(default)]
    pub _marker: std::marker::PhantomData<fn() -> B>,
}

impl<A, B> AgentImpl<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub fn as_mut_agent(&mut self) -> &mut A {
        &mut self.agent
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        Activatable, Agent, AgentId, AgentKind, AgentOrder, AgentStats, MobilityInfo, Movable,
        Orderable,
    };
    use crate::bucket::tests::MyBucket;
    use crate::bucket::TimeMS;
    use rand::random;
    use std::fmt::{Debug, Display, Formatter};

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
    pub(crate) enum DeviceType {
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
    pub(crate) struct DeviceStats {
        pub(crate) size: f32,
    }

    impl AgentStats for DeviceStats {}

    #[derive(Default, Clone, Debug)]
    pub(crate) struct TDevice {
        pub(crate) id: AgentId,
        pub(crate) device_type: DeviceType,
        pub(crate) order: AgentOrder,
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

        fn stage_one(&mut self, _core: &mut crate::core::Core<Self, MyBucket>) {
            self.stats.size = random();
        }

        fn stage_two_reverse(&mut self, _core: &mut crate::core::Core<Self, MyBucket>) {
            self.stats.size = 0.0;
        }
    }

    pub(crate) fn make_device(id: AgentId, device_type: DeviceType, order: i32) -> TDevice {
        TDevice {
            id,
            device_type,
            order: AgentOrder::from(order as u32),
            stats: Default::default(),
            step: TimeMS::default(),
        }
    }

    #[test]
    fn test_device_creation() {
        let device_a = make_device(AgentId::from(1), DeviceType::TypeA, 1);
        assert_eq!(device_a.id, AgentId::from(1));
    }

    #[test]
    fn test_device_comparison() {
        let device_a = make_device(AgentId::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(AgentId::from(2), DeviceType::TypeB, 2);
        assert_ne!(device_a.id, device_b.id);
        assert_ne!(device_a.device_type, device_b.device_type);
        assert_eq!(device_a.order, AgentOrder::from(1));
        assert_eq!(device_b.order, AgentOrder::from(2));
    }

    #[test]
    fn test_device_step() {
        let mut device_a = make_device(AgentId::from(1), DeviceType::TypeA, 1);
        let mut bucket = MyBucket::default();
        assert_eq!(bucket.step, TimeMS::default());
    }
}
