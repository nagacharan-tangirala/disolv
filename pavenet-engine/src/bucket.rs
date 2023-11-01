use crate::scheduler::Scheduler;
use std::ops::{Add, AddAssign, Mul};

/// A trait used to represent time stamps. Use this to define your own time stamp type.
/// All the engine parameters should be defined using this type.
pub trait TimeStamp:
    Default
    + Copy
    + AddAssign
    + Clone
    + Ord
    + Mul<Output = Self>
    + Add<Output = Self>
    + Send
    + Sync
    + From<u64>
    + 'static
{
    fn as_f32(&self) -> f32;
}

/// A trait passed to the entity so that an entity can access other entities. Any common models
/// applicable to all the entities irrespective of type should be assigned to a struct that
/// implements this trait.
pub trait Bucket<T>: Clone + Send + Sync + 'static
where
    T: TimeStamp,
{
    type SchedulerImpl: Scheduler<T>;

    fn scheduler(&mut self) -> &mut Self::SchedulerImpl;
    fn init(&mut self, step: T);
    fn update(&mut self, step: T);
    fn before_uplink(&mut self);
    fn after_downlink(&mut self);
    fn streaming_step(&mut self, step: T);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Bucket, TimeStamp};
    use crate::scheduler::tests::{make_scheduler_with_2_devices, MyScheduler};
    use std::fmt::Display;
    use std::ops::{Add, AddAssign, Mul};

    #[derive(Default, Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
    pub struct Ts(u32);

    impl AddAssign for Ts {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
        }
    }

    impl Mul for Ts {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(self.0 * rhs.0)
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

    impl Into<u64> for Ts {
        fn into(self) -> u64 {
            self.0 as u64
        }
    }

    impl TimeStamp for Ts {
        fn as_f32(&self) -> f32 {
            self.0 as f32
        }
    }

    impl Display for Ts {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Default, Clone)]
    pub(crate) struct MyBucket {
        pub(crate) scheduler: MyScheduler,
        pub(crate) step: Ts,
    }

    impl MyBucket {
        pub(crate) fn new() -> Self {
            let scheduler = make_scheduler_with_2_devices();
            Self {
                scheduler,
                step: Ts::default(),
            }
        }
    }

    impl Bucket<Ts> for MyBucket {
        type SchedulerImpl = MyScheduler;
        fn scheduler(&mut self) -> &mut MyScheduler {
            &mut self.scheduler
        }

        fn init(&mut self, step: Ts) {
            self.step = step;
        }

        fn update(&mut self, step: Ts) {
            self.step = step;
            println!("Update in MyBucket at {}", step);
        }

        fn before_uplink(&mut self) {
            println!("before_uplink in MyBucket");
        }

        fn after_downlink(&mut self) {
            println!("after_downlink in MyBucket");
        }

        fn streaming_step(&mut self, step: Ts) {
            println!("Streaming step in bucket at {}", step);
        }
    }

    #[test]
    fn test_bucket_update() {
        let mut bucket = MyBucket::default();
        let scheduler = MyScheduler::default();
        let step0 = Ts::from(0);
        bucket.init(step0);
        assert_eq!(bucket.step, Ts::from(0));
        let step1 = Ts::from(1);
        bucket.update(step1);
        assert_eq!(bucket.step, Ts::from(1));
        let step2 = Ts::from(2);
        bucket.update(step2);
        assert_eq!(bucket.step, Ts::from(2));
    }
}
