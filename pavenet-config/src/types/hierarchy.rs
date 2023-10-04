#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hierarchy(u32);

impl From<u32> for Hierarchy {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Into<i32> for Hierarchy {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<u32> for Hierarchy {
    fn into(self) -> u32 {
        self.0
    }
}
