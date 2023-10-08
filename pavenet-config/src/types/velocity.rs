use std::fmt::Display;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity(f32);

impl Display for Velocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for Velocity {
    fn from(f: f32) -> Self {
        Self(f as f32)
    }
}

impl Into<f32> for Velocity {
    fn into(self) -> f32 {
        self.0
    }
}
