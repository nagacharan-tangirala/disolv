#[derive(Debug, Clone, Copy)]
pub enum RadioMetrics {
    Latency,
    Throughput,
    PacketLoss,
}

pub mod latency {
    use pavenet_engine::channel::Metric;
    use serde::Deserialize;
    use std::ops::{Add, AddAssign, Mul, Sub};

    #[derive(Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
    pub struct Latency(u32);

    impl Latency {
        pub fn new(value: u32) -> Self {
            Self(value)
        }
    }

    impl From<f32> for Latency {
        fn from(value: f32) -> Self {
            Self(value as u32)
        }
    }

    impl Add for Latency {
        type Output = Self;

        fn add(self, other: Self) -> Self::Output {
            Self(self.0 + other.0)
        }
    }

    impl Sub for Latency {
        type Output = Self;

        fn sub(self, other: Self) -> Self::Output {
            Self(self.0 - other.0)
        }
    }

    impl Mul for Latency {
        type Output = Self;

        fn mul(self, other: Self) -> Self::Output {
            Self(self.0 * other.0)
        }
    }

    impl AddAssign for Latency {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Metric for Latency {
        fn as_f32(&self) -> f32 {
            self.0 as f32
        }
    }
}

pub mod throughput {
    use pavenet_engine::channel::Metric;
    use std::ops::{Add, AddAssign};

    #[derive(Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
    pub struct Throughput(u32);

    impl AddAssign for Throughput {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Add for Throughput {
        type Output = Self;

        fn add(self, other: Self) -> Self::Output {
            Self(self.0 + other.0)
        }
    }

    impl Metric for Throughput {
        fn as_f32(&self) -> f32 {
            self.0 as f32
        }
    }
}

pub mod bandwidth {
    use pavenet_engine::channel::Metric;
    use std::ops::{Add, AddAssign};

    #[derive(Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
    pub struct Bandwidth(u32);

    impl AddAssign for Bandwidth {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Add for Bandwidth {
        type Output = Self;

        fn add(self, other: Self) -> Self::Output {
            Self(self.0 + other.0)
        }
    }

    impl Metric for Bandwidth {
        fn as_f32(&self) -> f32 {
            self.0 as f32
        }
    }
}
