use crate::utils::constants::STREAM_TIME;
use krabmaga::engine::location::Real2D;

pub trait Mobility {
    fn location(&mut self, time_step: u64) -> Real2D;
}

struct StaticMobility {
    location: Real2D,
}

impl StaticMobility {
    pub fn new(location: Real2D) -> Self {
        Self { location }
    }
}

impl Mobility for StaticMobility {
    fn location(&mut self, _time_step: u64) -> Real2D {
        self.location
    }
}

#[derive(Clone, Copy)]
pub struct Trace {
    pub time_step: [u64; STREAM_TIME],
    pub location: [Real2D; STREAM_TIME],
    pub velocity: [f64; STREAM_TIME],
}

#[derive(Clone, Copy)]
pub struct TraceMobility {
    trace: Trace,
    current_index: usize,
}

impl TraceMobility {
    pub fn new(trace: Trace) -> Self {
        Self {
            trace,
            current_index: 0,
        }
    }
}

impl Mobility for TraceMobility {
    fn location(&mut self, time_step: u64) -> Real2D {
        let index = self
            .trace
            .time_step
            .iter()
            .position(|&x| x == time_step)
            .unwrap_or(self.current_index);

        self.current_index = index;
        self.trace.location[index]
    }
}
