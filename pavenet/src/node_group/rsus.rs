use crate::collections::collection::NodeCollection;
use crate::devices::rsu::RoadsideUnit;
use krabmaga::engine::schedule::Schedule;
use log::error;
use pavenet_config::config::types::{DeviceId, TimeStamp};
use pavenet_models::device::power::PowerState;

pub struct RoadsideUnits {
    rsus: Vec<RoadsideUnit>,
    pub devices_to_add: Vec<(DeviceId, TimeStamp)>,
    pub devices_to_pop: Vec<DeviceId>,
}

impl RoadsideUnits {
    pub(crate) fn before_step(&mut self, step: TimeStamp) {
        self.space.refresh_map(step);
    }

    pub(crate) fn update(&mut self, step: TimeStamp) {}
}

impl NodeCollection for RoadsideUnits {
    fn before_step(&mut self, step: TimeStamp) {
        self.space.refresh_map(step);
    }

    fn update(&mut self, step: TimeStamp) {}

    fn add_all(&mut self) {
        for rsu in self.rsus.iter_mut() {
            let time_stamp = rsu.models.power_schedule.pop_time_to_on();
            self.devices_to_add.push((rsu.id, time_stamp));
        }
    }

    fn power_off(&mut self, schedule: &mut Schedule) {
        for rsu_id in self.devices_to_pop.iter() {
            if let Some(rsu) = self.rsus.get_mut(rsu_id) {
                rsu.power_state = PowerState::Off;
                schedule.dequeue(Box::new(*rsu), rsu.id.into());
            } else {
                panic!("Rsu {} not found", rsu_id);
            }
        }
    }

    fn schedule_power_on(&mut self, schedule: &mut Schedule) {
        for rsu_ts in self.devices_to_add.iter() {
            if let Some(rsu) = self.rsus.get_mut(&rsu_ts.0) {
                if !schedule.schedule_repeating(Box::new(*rsu), rsu.id.into(), rsu_ts.1 as f32, 0) {
                    error!("Could not schedule rsu {} ", rsu.id);
                    panic!("Could not schedule rsu {} ", rsu.id);
                }
            }
        }
    }
}
