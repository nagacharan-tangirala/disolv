use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub enum PowerState {
    #[default]
    Off,
    On,
}

impl Display for PowerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerState::On => write!(f, "On"),
            PowerState::Off => write!(f, "Off"),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, TypedBuilder)]
pub struct PowerManager {
    pub on_times: VecDeque<TimeMS>,
    pub off_times: VecDeque<TimeMS>,
    array_idx: usize,
    #[builder(default)]
    pub on_since: TimeMS,
}

impl PowerManager {
    pub fn peek_time_to_off(&self) -> TimeMS {
        match self.off_times.front() {
            Some(time_stamp) => *time_stamp,
            None => TimeMS::default(),
        }
    }

    pub fn has_next_time_to_on(&self) -> bool {
        self.array_idx < self.on_times.len()
    }

    pub fn pop_time_to_on(&mut self) -> TimeMS {
        self.on_since = match self.on_times.front() {
            Some(time_stamp) => *time_stamp,
            None => TimeMS::default(),
        };
        self.on_times.pop_front().unwrap_or_default()
    }

    pub fn pop_time_to_off(&mut self) {
        self.off_times.pop_front();
    }
}
