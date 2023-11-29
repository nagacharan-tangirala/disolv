use pavenet_engine::bucket::TimeS;
use pavenet_input::power::data::PowerTimes;
use std::collections::VecDeque;

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub enum PowerState {
    #[default]
    Off,
    On,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct PowerManager {
    pub on_times: VecDeque<TimeS>,
    pub off_times: VecDeque<TimeS>,
    array_idx: usize,
}

impl PowerManager {
    pub fn new(power_times: PowerTimes) -> Self {
        Self {
            on_times: power_times.0.into(),
            off_times: power_times.1.into(),
            array_idx: 0,
        }
    }

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
