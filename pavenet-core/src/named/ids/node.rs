use serde_derive::Deserialize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:09}", self.0)
    }
}

impl FromStr for NodeId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>()?;
        Ok(Self(id))
    }
}

impl Into<u32> for NodeId {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<i64> for NodeId {
    fn from(f: i64) -> Self {
        Self(f as u32)
    }
}

impl NodeId {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
