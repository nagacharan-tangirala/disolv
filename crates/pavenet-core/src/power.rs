use pavenet_engine::bucket::TimeMS;
use std::collections::VecDeque;
use typed_builder::TypedBuilder;

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub enum PowerState {
    #[default]
    Off,
    On,
}

#[derive(Clone, Default, Debug, PartialEq, TypedBuilder)]
pub struct PowerManager {
    pub on_times: VecDeque<TimeMS>,
    pub off_times: VecDeque<TimeMS>,
    array_idx: usize,
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
        self.on_times
            .pop_front()
            .unwrap_or_else(|| TimeMS::default())
    }

    pub fn pop_time_to_off(&mut self) {
        self.off_times.pop_front();
    }
}
