#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order(i32);

impl From<i32> for Order {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Into<u32> for Order {
    fn into(self) -> u32 {
        self.0 as u32
    }
}

impl Order {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}
