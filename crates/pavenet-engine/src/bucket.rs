use crate::scheduler::Scheduler;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul};
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeS(pub u64);

impl Display for TimeS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:09}", self.0)
    }
}

impl FromStr for TimeS {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for TimeS {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl From<i32> for TimeS {
    fn from(f: i32) -> Self {
        Self(f as u64)
    }
}

impl From<i64> for TimeS {
    fn from(f: i64) -> Self {
        Self(f as u64)
    }
}

impl TimeS {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

impl Mul for TimeS {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for TimeS {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for TimeS {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for TimeS {
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
    fn init(&mut self, step: TimeS);
    fn update(&mut self, step: TimeS);
    fn before_uplink(&mut self);
    fn after_downlink(&mut self);
    fn streaming_step(&mut self, step: TimeS);
}

/// The <code>ResultSaver</code> trait defines the methods that take the simulator data and
/// prepare the data for output.
pub trait ResultSaver: Bucket {
    fn save_device_stats(&mut self, step: TimeS);
    fn save_data_stats(&mut self, step: TimeS);
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
    use super::TimeS;
    use crate::scheduler::tests::{make_scheduler_with_2_devices, MyScheduler};
    use std::fmt::Display;

    #[derive(Default, Clone)]
    pub(crate) struct MyBucket {
        pub(crate) scheduler: MyScheduler,
        pub(crate) step: TimeS,
    }

    impl MyBucket {
        pub(crate) fn new() -> Self {
            let scheduler = make_scheduler_with_2_devices();
            Self {
                scheduler,
                step: TimeS::default(),
            }
        }
    }

    impl ResultSaver for MyBucket {
        fn save_device_stats(&mut self, time: TimeS) {
            todo!()
        }
        fn save_data_stats(&mut self, time: TimeS) {
            todo!()
        }
    }

    impl Bucket for MyBucket {
        type SchedulerImpl = MyScheduler;
        fn scheduler(&mut self) -> &mut MyScheduler {
            &mut self.scheduler
        }

        fn init(&mut self, step: TimeS) {
            self.step = step;
        }

        fn update(&mut self, step: TimeS) {
            self.step = step;
            println!("Update in MyBucket at {}", step);
        }

        fn before_uplink(&mut self) {
            println!("before_uplink in MyBucket");
        }

        fn after_downlink(&mut self) {
            println!("after_downlink in MyBucket");
        }

        fn streaming_step(&mut self, step: TimeS) {
            println!("Streaming step in bucket at {}", step);
        }
    }

    #[test]
    fn test_bucket_update() {
        let mut bucket = MyBucket::default();
        let scheduler = MyScheduler::default();
        let step0 = TimeS::from(0i64);
        bucket.init(step0);
        assert_eq!(bucket.step, TimeS::from(0i64));
        let step1 = TimeS::from(1i64);
        bucket.update(step1);
        assert_eq!(bucket.step, TimeS::from(1i64));
        let step2 = TimeS::from(2i64);
        bucket.update(step2);
        assert_eq!(bucket.step, TimeS::from(2i64));
    }
}
