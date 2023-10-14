use krabmaga::engine::schedule::Schedule;
use std::ops::{Add, AddAssign};

pub trait TimeS:
    Default + Copy + AddAssign + Clone + Ord + Add + Send + Sync + From<u64> + 'static
{
}

pub trait Bucket<S>: Clone + Send + Sync + 'static
where
    S: TimeS,
{
    fn init(&mut self, schedule: &mut Schedule);
    fn before_step(&mut self, schedule: &mut Schedule);
    fn update(&mut self, step: S);
    fn after_step(&mut self, schedule: &mut Schedule);
    fn streaming_step(&mut self, step: S);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Bucket, TimeS};
    use crate::entity::tests::Nid;
    use crate::node::tests::MyNode;
    use krabmaga::engine::schedule::Schedule;
    use std::collections::HashMap;
    use std::fmt::Display;
    use std::ops::{Add, AddAssign};

    #[derive(Default, Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
    pub struct Ts(u32);

    impl AddAssign for Ts {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
        }
    }

    impl Add for Ts {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    impl From<u64> for Ts {
        fn from(value: u64) -> Self {
            Self(value as u32)
        }
    }

    impl Into<f32> for Ts {
        fn into(self) -> f32 {
            self.0 as f32
        }
    }

    impl Into<u64> for Ts {
        fn into(self) -> u64 {
            self.0 as u64
        }
    }

    impl TimeS for Ts {}

    impl Display for Ts {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Default, Clone)]
    pub(crate) struct MyBucket {
        pub(crate) step: Ts,
        pub(crate) devices: HashMap<Nid, MyNode>,
    }

    impl MyBucket {
        pub(crate) fn add(&mut self, node: MyNode) {
            self.devices.insert(node.node.id, node);
        }

        pub(crate) fn add_to_schedule(&mut self, schedule: &mut Schedule) {
            for (_, node) in self.devices.iter_mut() {
                schedule.schedule_repeating(
                    Box::new(node.clone()),
                    node.node_id.into(),
                    0.,
                    node.node.order,
                );
            }
        }
    }

    impl Bucket<Ts> for MyBucket {
        fn init(&mut self, schedule: &mut Schedule) {
            self.add_to_schedule(schedule);
            self.step = Ts::default();
        }

        fn before_step(&mut self, _schedule: &mut Schedule) {
            println!("Before step in MyBucket");
        }

        fn update(&mut self, step: Ts) {
            self.step = step;
            println!("Update in MyBucket at {}", step);
        }

        fn after_step(&mut self, _schedule: &mut Schedule) {
            println!("After step in MyBucket");
        }

        fn streaming_step(&mut self, step: Ts) {
            println!("Streaming step in bucket at {}", step);
        }
    }

    #[test]
    fn test_bucket_init() {
        let mut schedule = Schedule::new();
        let mut bucket = MyBucket::default();
        bucket.init(&mut schedule);
        assert_eq!(bucket.step, Ts::default());
    }

    #[test]
    fn test_bucket_update() {
        let mut schedule = Schedule::new();
        let mut bucket = MyBucket::default();
        bucket.init(&mut schedule);
        bucket.update(Ts::from(1));
        assert_eq!(bucket.step, Ts::from(1));
    }
}
