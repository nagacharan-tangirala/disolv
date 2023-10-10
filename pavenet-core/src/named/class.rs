use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Hash, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class(u32);

impl From<u32> for Class {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Into<i32> for Class {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Class {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
