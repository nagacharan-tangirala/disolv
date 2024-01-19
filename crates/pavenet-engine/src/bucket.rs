use crate::scheduler::Scheduler;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul};
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeMS(pub u64);

impl Display for TimeMS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TimeMS {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for TimeMS {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl From<i32> for TimeMS {
    fn from(f: i32) -> Self {
        Self(f as u64)
    }
}

impl From<i64> for TimeMS {
    fn from(f: i64) -> Self {
        Self(f as u64)
    }
}

impl TimeMS {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

impl Mul for TimeMS {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for TimeMS {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for TimeMS {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for TimeMS {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// A trait passed to the entity so that an entity can access other entities. Any common models
/// applicable to all the entities irrespective of type should be assigned to a struct that
/// implements this trait.
pub trait Bucket: Clone + Send + Sync + 'static {
    type SchedulerImpl: Scheduler;

    fn scheduler(&mut self) -> &mut Self::SchedulerImpl;
    fn init(&mut self, step: TimeMS);
    fn update(&mut self, step: TimeMS);
    fn before_uplink(&mut self);
    fn before_downlink(&mut self);
    fn streaming_step(&mut self, step: TimeMS);
    fn end(&mut self, step: TimeMS);
}

/// The <code>ResultSaver</code> trait defines the methods that take the simulator data and
/// prepare the data for output.
pub trait ResultSaver: Bucket {
    fn save_device_stats(&mut self, step: TimeMS);
    fn save_data_stats(&mut self, step: TimeMS);
    fn save_network_stats(&mut self, step: TimeMS);
}

/// The <code>Resultant</code> trait marks data that can be written as output. Use this to mark
/// a struct which contains the data that needs to be written to a file.
pub trait Resultant: Serialize + Copy + Clone + Debug {}

pub trait Outlet<R>
where
    R: Resultant,
{
    fn write_to_file(&mut self, data: &R);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Bucket;
    use super::ResultSaver;
    use super::TimeMS;
    use crate::scheduler::tests::{make_scheduler_with_2_devices, MyScheduler};

    #[derive(Default, Clone)]
    pub(crate) struct MyBucket {
        pub(crate) scheduler: MyScheduler,
        pub(crate) step: TimeMS,
    }

    impl MyBucket {
        pub(crate) fn new() -> Self {
            let scheduler = make_scheduler_with_2_devices();
            Self {
                scheduler,
                step: TimeMS::default(),
            }
        }
    }

    impl ResultSaver for MyBucket {
        fn save_device_stats(&mut self, time: TimeMS) {
            todo!()
        }
        fn save_data_stats(&mut self, time: TimeMS) {
            todo!()
        }

        fn save_network_stats(&mut self, step: TimeMS) {
            todo!()
        }
    }

    impl Bucket for MyBucket {
        type SchedulerImpl = MyScheduler;
        fn scheduler(&mut self) -> &mut MyScheduler {
            &mut self.scheduler
        }

        fn init(&mut self, step: TimeMS) {
            self.step = step;
        }

        fn update(&mut self, step: TimeMS) {
            self.step = step;
            println!("Update in MyBucket at {}", step);
        }

        fn before_uplink(&mut self) {
            println!("before_uplink in MyBucket");
        }

        fn before_downlink(&mut self) {
            println!("before_downlink in MyBucket");
        }

        fn streaming_step(&mut self, step: TimeMS) {
            println!("Streaming step in bucket at {}", step);
        }

        fn end(&mut self, step: TimeMS) {
            println!("End in MyBucket at {}", step);
        }
    }

    #[test]
    fn test_bucket_update() {
        let mut bucket = MyBucket::default();
        let scheduler = MyScheduler::default();
        let step0 = TimeMS::from(0i64);
        bucket.init(step0);
        assert_eq!(bucket.step, TimeMS::from(0i64));
        let step1 = TimeMS::from(1i64);
        bucket.update(step1);
        assert_eq!(bucket.step, TimeMS::from(1i64));
        let step2 = TimeMS::from(2i64);
        bucket.update(step2);
        assert_eq!(bucket.step, TimeMS::from(2i64));
    }
}
