use super::bucket::{Bucket, TimeS};
use std::hash::Hash;

pub trait Identifier:
    Default + Clone + Copy + Hash + PartialEq + Eq + Send + Sync + 'static
{
}

pub trait Kind: Default + Clone + Copy + PartialEq + Eq + Send + Sync + 'static {}

pub trait Entity<B, S>: Default + Clone + Send + Sync + 'static
where
    B: Bucket<S>,
    S: TimeS,
{
    fn step(&mut self, bucket: &mut B);
    fn after_step(&mut self, bucket: &mut B);
    fn is_stopped(&self) -> bool;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Entity, Identifier, Kind};
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::node::tests::as_node;
    use krabmaga::engine::schedule::Schedule;
    use std::fmt::Display;

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Nid(u32);

    impl Identifier for Nid {}

    impl std::fmt::Display for Nid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) enum DeviceType {
        #[default]
        TypeA,
        TypeB,
    }

    impl Kind for DeviceType {}

    impl Display for DeviceType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DeviceType::TypeA => write!(f, "TypeA"),
                DeviceType::TypeB => write!(f, "TypeB"),
            }
        }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub(crate) struct TDevice {
        pub(crate) id: Nid,
        pub(crate) device_type: DeviceType,
        pub(crate) order: i32,
        pub(crate) step: Ts,
    }

    impl Entity<MyBucket, Ts> for TDevice {
        fn step(&mut self, bucket: &mut MyBucket) {
            self.step = bucket.step;
            println!("step {} in TDevice of type {}", self.step, self.device_type);
        }
        fn after_step(&mut self, _bucket: &mut MyBucket) {
            println!("after_step in TDevice of type {}", self.device_type);
        }
        fn is_stopped(&self) -> bool {
            false
        }
    }

    pub(crate) fn make_device(id: Nid, device_type: DeviceType, order: i32) -> TDevice {
        TDevice {
            id,
            device_type,
            order,
            step: Ts::default(),
        }
    }

    #[test]
    fn test_device_creation() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        assert_eq!(device_a.id, Nid::from(1));
    }

    #[test]
    fn test_device_comparison() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(Nid::from(2), DeviceType::TypeB, 2);
        assert_ne!(device_a.id, device_b.id);
        assert_ne!(device_a.device_type, device_b.device_type);
        assert_eq!(device_a.order, 1);
        assert_eq!(device_b.order, 2);
    }

    #[test]
    fn test_device_step() {
        let mut device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let mut bucket = MyBucket::default();
        device_a.step(&mut bucket);
        assert_eq!(bucket.step, Ts::default());
    }

    #[test]
    fn test_add_to_bucket() {
        let mut bucket = MyBucket::default();
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(Nid::from(2), DeviceType::TypeB, 2);
        let node_a = as_node(device_a);
        let node_b = as_node(device_b);
        bucket.add(node_a);
        bucket.add(node_b);
        assert_eq!(bucket.devices.len(), 2);
    }

    #[test]
    fn test_add_to_schedule() {
        let mut schedule = Schedule::new();
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        let x = schedule.schedule_repeating(
            Box::new(node_a.clone()),
            device_a.id.into(),
            Ts::default().into(),
            0,
        );
        assert_eq!(x, true);
    }
}
