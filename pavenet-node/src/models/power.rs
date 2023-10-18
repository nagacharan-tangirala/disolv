use pavenet_recipe::times::ts::TimeS;

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub enum PowerState {
    #[default]
    Off,
    On,
}

pub const SCHEDULE_SIZE: usize = 20;

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub struct PowerSchedule {
    pub on_times: [Option<TimeS>; SCHEDULE_SIZE],
    pub off_times: [Option<TimeS>; SCHEDULE_SIZE],
    array_idx: usize,
}

impl PowerSchedule {
    pub fn new(
        on_times: [Option<TimeS>; SCHEDULE_SIZE],
        off_times: [Option<TimeS>; SCHEDULE_SIZE],
    ) -> Self {
        Self {
            on_times,
            off_times,
            array_idx: 0,
        }
    }

    pub fn peek_time_to_off(self) -> TimeS {
        if self.array_idx == SCHEDULE_SIZE {
            return TimeS::default();
        }
        match self.off_times[self.array_idx] {
            Some(time_stamp) => time_stamp,
            None => TimeS::default(),
        }
    }

    pub fn pop_time_to_on(&mut self) -> TimeS {
        if self.array_idx == SCHEDULE_SIZE {
            return TimeS::default();
        }
        match self.on_times[self.array_idx] {
            Some(time_stamp) => {
                self.on_times[self.array_idx] = None;
                time_stamp
            }
            None => TimeS::default(),
        }
    }

    pub fn pop_time_to_off(&mut self) {
        if self.array_idx < SCHEDULE_SIZE {
            self.off_times[self.array_idx] = None;
            self.array_idx += 1;
        }
    }
}
