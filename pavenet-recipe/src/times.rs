pub mod ts {
    use pavenet_engine::bucket::TimeStamp;
    use serde_derive::Deserialize;
    use std::fmt::Display;
    use std::ops::{Add, AddAssign};
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

    impl From<i64> for TimeS {
        fn from(f: i64) -> Self {
            Self(f as u64)
        }
    }

    impl Into<u64> for TimeS {
        fn into(self) -> u64 {
            self.0
        }
    }

    impl Into<f32> for TimeS {
        fn into(self) -> f32 {
            self.0 as f32
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

    impl TimeStamp for TimeS {}
}

pub mod ms {
    use std::fmt::Display;
    use std::str::FromStr;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MilliSeconds(u64);

    impl Display for MilliSeconds {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:09}", self.0)
        }
    }

    impl FromStr for MilliSeconds {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let id = s.parse::<u64>()?;
            Ok(Self(id))
        }
    }

    impl From<u64> for MilliSeconds {
        fn from(f: u64) -> Self {
            Self(f)
        }
    }

    impl From<i64> for MilliSeconds {
        fn from(f: i64) -> Self {
            Self(f as u64)
        }
    }

    impl Into<u64> for MilliSeconds {
        fn into(self) -> u64 {
            self.0
        }
    }
}
