use crate::data::data_io::TimeStamp;
use crate::utils::constants::ARRAY_SIZE;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DeviceState {
    Inactive = 0,
    Active,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct Timing {
    pub(crate) activation: [Option<TimeStamp>; ARRAY_SIZE],
    pub(crate) deactivation: [Option<TimeStamp>; ARRAY_SIZE],
    array_idx: usize,
}

impl Timing {
    pub(crate) fn new(
        activation: [Option<TimeStamp>; ARRAY_SIZE],
        deactivation: [Option<TimeStamp>; ARRAY_SIZE],
    ) -> Self {
        Self {
            activation,
            deactivation: deactivation,
            array_idx: 0,
        }
    }

    pub(crate) fn peek_activation_time(self) -> TimeStamp {
        if self.array_idx == ARRAY_SIZE {
            return 0;
        }
        match self.activation[self.array_idx] {
            Some(time_stamp) => time_stamp,
            None => 0,
        }
    }

    pub(crate) fn peek_deactivation_time(self) -> TimeStamp {
        if self.array_idx == ARRAY_SIZE {
            return 0;
        }
        match self.deactivation[self.array_idx] {
            Some(time_stamp) => time_stamp,
            None => 0,
        }
    }

    pub(crate) fn pop_activation_time(&mut self) -> TimeStamp {
        if self.array_idx == ARRAY_SIZE {
            return 0;
        }
        match self.activation[self.array_idx] {
            Some(time_stamp) => {
                self.activation[self.array_idx] = None;
                time_stamp
            }
            None => 0,
        }
    }

    pub(crate) fn pop_deactivation_time(&mut self) -> TimeStamp {
        if self.array_idx == ARRAY_SIZE {
            return 0;
        }
        match self.deactivation[self.array_idx] {
            Some(time_stamp) => {
                self.array_idx += 1;
                self.deactivation[self.array_idx] = None;
                time_stamp
            }
            None => 0,
        }
    }
}
