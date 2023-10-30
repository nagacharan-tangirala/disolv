use super::bucket::{Bucket, TimeStamp};
use std::fmt::Display;
use std::hash::Hash;

/// Unique value that identifies an entity. Only a single instance of this type
/// is allowed. Irrespective of the type of the entity, it should be uniquely identified
/// by a type implementing this trait.
/// For example, in a vehicular scenario, both the vehicles and the RSUs can be identified by a
/// single type (e.g. ID), which implements this trait.
pub trait Identifier:
    Default + Display + Clone + Copy + Hash + PartialEq + Eq + Send + Sync + 'static
{
    fn as_u32(&self) -> u32;
}

/// A trait that represents the tier of an entity. Extend this to a custom type that represents
/// the tier of your entity. Only one instance of this type is allowed. A named type masking an
/// integer is sufficient.
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
pub trait Tiered<T>
where
    T: Tier,
{
    fn tier(&self) -> T;
    fn set_tier(&mut self, tier: T);
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
pub trait Movable<B, M, T>
where
    B: Bucket<T>,
    M: MobilityInfo,
    T: TimeStamp,
{
    fn mobility(&self) -> &M;
    fn set_mobility(&mut self, bucket: &mut B);
}

/// A trait that allows an entity to be scheduled for simulation.
pub trait Schedulable<T>
where
    T: TimeStamp,
{
    fn stop(&mut self);
    fn is_stopped(&self) -> bool;
    fn time_to_add(&mut self) -> T;
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
pub trait Entity<B, T, Ts>: Schedulable<Ts> + Tiered<T> + Clone + Send + Sync + 'static
where
    B: Bucket<Ts>,
    T: Tier,
    Ts: TimeStamp,
{
    fn uplink_stage(&mut self, bucket: &mut B);
    fn downlink_stage(&mut self, bucket: &mut B);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Entity, Identifier, Kind, MobilityInfo, Movable, Schedulable, Tier, Tiered};
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::bucket::TimeStamp;
    use crate::engine::tests::as_node;
    use krabmaga::engine::schedule::Schedule;
    use std::fmt::{Debug, Display, Formatter};

    #[derive(Copy, Clone, Default, Debug)]
    struct Mobility {
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

    #[derive(Copy, Clone, Default, Debug)]
    struct Device {
        mobility: Mobility,
    }

    impl Movable<MyBucket, Mobility, Ts> for Device {
        fn mobility(&self) -> &Mobility {
            &self.mobility
        }

        fn set_mobility(&mut self, my_bucket: &mut MyBucket) {
            todo!("Read from bucket");
        }
    }

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Nid(u32);

    impl Identifier for Nid {
        fn as_u32(&self) -> u32 {
            self.0
        }
    }

    impl Display for Nid {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<i32> for Nid {
        fn from(value: i32) -> Self {
            Self(value as u32)
        }
    }

    impl Into<u32> for Nid {
        fn into(self) -> u32 {
            self.0
        }
    }

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
        pub(crate) id: Nid,
        pub(crate) device_type: DeviceType,
        pub(crate) order: Level,
        pub(crate) step: Ts,
    }

    impl Schedulable<Ts> for TDevice {
        fn stop(&mut self) {}

        fn is_stopped(&self) -> bool {
            false
        }

        fn time_to_add(&mut self) -> Ts {
            Ts::from(0)
        }
    }

    impl Tiered<Level> for TDevice {
        fn tier(&self) -> Level {
            Level::from(self.order.as_i32() as u32)
        }

        fn set_tier(&mut self, tier: Level) {
            self.order = tier;
        }
    }

    impl Entity<MyBucket, Level, Ts> for TDevice {
        fn uplink_stage(&mut self, bucket: &mut MyBucket) {
            self.step = bucket.step;
            println!("step {} in TDevice of type {}", self.step, self.device_type);
        }

        fn downlink_stage(&mut self, _bucket: &mut MyBucket) {
            println!("after_step in TDevice of type {}", self.device_type);
        }
    }

    pub(crate) fn make_device(id: Nid, device_type: DeviceType, order: i32) -> TDevice {
        TDevice {
            id,
            device_type,
            order: Level::from(order as u32),
            step: Ts::default(),
        }
    }

    #[test]
    fn test_device_creation() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        assert_eq!(device_a.id, Nid::from(1));
    }

    #[test]
    fn test_ts_addition() {
        let mut ts = Ts::from(1);
        ts += Ts::from(1);
        assert_eq!(ts, Ts::from(2));
    }

    #[test]
    fn test_device_comparison() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(Nid::from(2), DeviceType::TypeB, 2);
        assert_ne!(device_a.id, device_b.id);
        assert_ne!(device_a.device_type, device_b.device_type);
        assert_eq!(device_a.order, Level::from(1));
        assert_eq!(device_b.order, Level::from(2));
    }

    #[test]
    fn test_device_step() {
        let mut device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let mut bucket = MyBucket::default();
        device_a.uplink_stage(&mut bucket);
        assert_eq!(bucket.step, Ts::default());
    }

    #[test]
    fn test_add_to_schedule() {
        let mut schedule = Schedule::new();
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        let x = schedule.schedule_repeating(
            Box::new(node_a.clone()),
            node_a.node_id.into(),
            Ts::default().as_f32(),
            0,
        );
        assert_eq!(x, true);
    }
}
