use super::bucket::Bucket;
use super::bucket::TimeS;
use std::fmt::Display;
use std::hash::Hash;

/// A trait that represents the tier of an entity. Extend this to a custom type that represents
/// the tier of your entity. Only one instance of this type is allowed. A named type masking an
/// integer is sufficient or an enum is also sufficient.
///
/// This is required to control the order of calling the uplink and downlink stages of the
/// entities. At each time step, the entities are sorted by their tier. The entities with the
/// lowest tier are called first and gradually proceeding to the entities with the highest tier.
/// This allows the entities to be simulated in a tiered fashion.
pub trait Tier: Default + Copy + Clone + Hash + PartialEq + Eq + Send + Sync + 'static {
    fn as_i32(&self) -> i32;
}

/// Trait that represents the kind of an entity. Extend this to a custom type
/// (e.g. enum) that represents the kind of your entity. Only one instance of this type
/// is allowed. This is required to distinguish between different types of entities.
///
/// Multiple types of entities can be simulated in a single simulation. However, all entities
/// must be distinguishable by only their kind. For example, in a vehicular scenario, this trait
/// can be implemented for both vehicles and RSUs. There can also be multiple types of vehicles
/// (e.g. cars, trucks, buses, etc.). Similarly, there can be multiple types of RSUs (e.g. RSUs
/// with different transmission ranges). Each of these types can have their own struct that
/// implements the [Entity] trait. However, all these types must be documented in a single enum.
/// Such enum should implement this trait.
pub trait Kind:
    Default + Display + Clone + Copy + PartialEq + Eq + Hash + Send + Sync + 'static
{
}

/// A trait to get and set the tier of an entity.
pub trait Tiered {
    type T: Tier;
    fn tier(&self) -> Self::T;
    fn set_tier(&mut self, tier: Self::T);
}

/// A trait that represents the mobility information of an entity. Extend this to
/// a custom type that represents the static or dynamic positional information of entities.
///
/// Multiple types of mobility information can be used in a single simulation.
/// For example, static devices need only the positional information, while mobile devices need
/// both the positional and mobility information.
pub trait MobilityInfo: Copy + Clone {}

/// A trait to get and set the mobility information of an entity. Must extend this for
/// both the static and mobile entities.
pub trait Movable<B>
where
    B: Bucket,
{
    type M: MobilityInfo;
    fn mobility(&self) -> &Self::M;
    fn set_mobility(&mut self, bucket: &mut B);
}

/// A trait that allows an entity to be scheduled for simulation.
pub trait Schedulable {
    fn stop(&mut self);
    fn is_stopped(&self) -> bool;
    fn time_to_add(&mut self) -> TimeS;
}

/// A trait that represents an entity. Extend this to a custom device type (e.g. struct) that
/// you want to simulate. Only types with this trait can be added to a bucket and hence
/// scheduled for simulation.
///
/// At each time step, the entities are called based on the [tier]. There are two passes for
/// each entity. In the first pass, <code>uplink_stage</code> is called on all the entities in
/// ascending order of their tier. In the second pass, <code>downlink_stage</code> is called on
/// all the entities in descending order of their tier. This allows the entities to be simulated
/// in a tiered fashion.
///
/// Starting at this trait will guide you to the other traits that you need to implement for the
/// device to be simulation-ready.
/// [tier]: Tier
pub trait Entity<B, T>: Schedulable + Tiered + Movable<B> + Clone + Send + Sync + 'static
where
    B: Bucket,
    T: Tier,
{
    fn uplink_stage(&mut self, bucket: &mut B);
    fn downlink_stage(&mut self, bucket: &mut B);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Entity, Kind, MobilityInfo, Movable, Schedulable, Tier, Tiered};
    use crate::bucket::tests::MyBucket;
    use crate::bucket::TimeS;
    use crate::engine::tests::as_node;
    use crate::node::NodeId;
    use krabmaga::engine::schedule::Schedule;
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

    #[derive(Default, Debug, Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
    pub(crate) struct Level(u32);

    impl From<u32> for Level {
        fn from(level: u32) -> Self {
            Self(level)
        }
    }

    impl Into<i32> for Level {
        fn into(self) -> i32 {
            self.0 as i32
        }
    }

    impl Tier for Level {
        fn as_i32(&self) -> i32 {
            self.0 as i32
        }
    }

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) enum DeviceType {
        #[default]
        TypeA,
        TypeB,
    }

    impl Kind for DeviceType {}

    impl Display for DeviceType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                DeviceType::TypeA => write!(f, "TypeA"),
                DeviceType::TypeB => write!(f, "TypeB"),
            }
        }
    }

    #[derive(Default, Clone, Debug)]
    pub(crate) struct TDevice {
        pub(crate) id: NodeId,
        pub(crate) device_type: DeviceType,
        pub(crate) order: Level,
        pub(crate) step: TimeS,
    }

    impl Schedulable for TDevice {
        fn stop(&mut self) {}

        fn is_stopped(&self) -> bool {
            false
        }

        fn time_to_add(&mut self) -> TimeS {
            TimeS::from(0)
        }
    }

    impl Tiered for TDevice {
        type T = Level;
        fn tier(&self) -> Level {
            Level::from(self.order.as_i32() as u32)
        }

        fn set_tier(&mut self, tier: Level) {
            self.order = tier;
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

    impl Entity<MyBucket, Level> for TDevice {
        fn uplink_stage(&mut self, bucket: &mut MyBucket) {
            self.step = bucket.step;
            println!("step {} in TDevice of type {}", self.step, self.device_type);
        }

        fn downlink_stage(&mut self, _bucket: &mut MyBucket) {
            println!("after_step in TDevice of type {}", self.device_type);
        }
    }

    pub(crate) fn make_device(id: NodeId, device_type: DeviceType, order: i32) -> TDevice {
        TDevice {
            id,
            device_type,
            order: Level::from(order as u32),
            step: TimeS::default(),
        }
    }

    #[test]
    fn test_device_creation() {
        let device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        assert_eq!(device_a.id, NodeId::from(1));
    }

    #[test]
    fn test_ts_addition() {
        let mut ts = TimeS::from(1);
        ts += TimeS::from(1);
        assert_eq!(ts, TimeS::from(2));
    }

    #[test]
    fn test_device_comparison() {
        let device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(NodeId::from(2), DeviceType::TypeB, 2);
        assert_ne!(device_a.id, device_b.id);
        assert_ne!(device_a.device_type, device_b.device_type);
        assert_eq!(device_a.order, Level::from(1));
        assert_eq!(device_b.order, Level::from(2));
    }

    #[test]
    fn test_device_step() {
        let mut device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        let mut bucket = MyBucket::default();
        device_a.uplink_stage(&mut bucket);
        assert_eq!(bucket.step, TimeS::default());
    }

    #[test]
    fn test_add_to_schedule() {
        let mut schedule = Schedule::new();
        let device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        let x = schedule.schedule_repeating(
            Box::new(node_a.clone()),
            node_a.node_id.as_u32(),
            TimeS::default().as_f32(),
            0,
        );
        assert_eq!(x, true);
    }
}
