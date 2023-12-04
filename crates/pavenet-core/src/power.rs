use pavenet_engine::bucket::TimeS;
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
    pub on_times: VecDeque<TimeS>,
    pub off_times: VecDeque<TimeS>,
    array_idx: usize,
}

impl PowerManager {
    pub fn peek_time_to_off(&self) -> TimeS {
        match self.off_times.front() {
            Some(time_stamp) => *time_stamp,
            None => TimeS::default(),
        }
    }

    pub fn pop_time_to_on(&mut self) -> TimeS {
        match self.on_times.pop_front() {
            Some(time_stamp) => time_stamp,
            None => TimeS::default(),
        }
    }

    pub fn pop_time_to_off(&mut self) {
        self.off_times.pop_front();
    }
}
