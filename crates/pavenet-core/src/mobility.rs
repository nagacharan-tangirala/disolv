use crate::mobility::road::RoadId;
use crate::mobility::velocity::Velocity;
use pavenet_engine::entity::MobilityInfo;
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug, Clone, Default)]
pub enum MobilityType {
    #[default]
    Stationery,
    Mobile,
}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct MapState {
    pub pos: Point2D,
    #[builder(default = None)]
    pub z: Option<f64>,
    #[builder(default = None)]
    pub velocity: Option<Velocity>,
    #[builder(default = None)]
    pub road_id: Option<RoadId>,
}

impl MobilityInfo for MapState {}

pub mod road {
    use std::fmt::Display;
    use std::str::FromStr;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct RoadId(u32);

    impl Display for RoadId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl FromStr for RoadId {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let id = s.parse::<u32>()?;
            Ok(Self(id))
        }
    }

    impl From<u32> for RoadId {
        fn from(f: u32) -> Self {
            Self(f)
        }
    }

    impl From<i64> for RoadId {
        fn from(f: i64) -> Self {
            Self(f as u32)
        }
    }

    impl RoadId {
        pub fn as_u32(&self) -> u32 {
            self.0
        }
        pub fn as_i64(&self) -> i64 {
            self.0 as i64
        }
    }
}

pub mod velocity {
    use std::fmt::Display;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Velocity(f32);

    impl Display for Velocity {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<f64> for Velocity {
        fn from(f: f64) -> Self {
            Self(f as f32)
        }
    }

    impl From<f32> for Velocity {
        fn from(f: f32) -> Self {
            Self(f)
        }
    }

    impl Velocity {
        pub fn as_f32(&self) -> f32 {
            self.0
        }
        pub fn as_f64(&self) -> f64 {
            self.0 as f64
        }
    }
}

pub mod cell {
    use std::fmt::Display;
    use std::str::FromStr;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct CellId(u32);

    impl Display for CellId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl FromStr for CellId {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let id = s.parse::<u32>()?;
            Ok(Self(id))
        }
    }

    impl From<f32> for CellId {
        fn from(f: f32) -> Self {
            Self(f as u32)
        }
    }

    impl From<f64> for CellId {
        fn from(f: f64) -> Self {
            Self(f as u32)
        }
    }

    impl CellId {
        pub fn as_u32(&self) -> u32 {
            self.0
        }
    }
}
