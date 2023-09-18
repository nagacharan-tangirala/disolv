use core::fmt;
use std::hash::{Hash, Hasher};

use crate::device::device_state::{DeviceState, Timing};
use crate::models::aggregator::{AggregatorType, BasicAggregator};
use crate::reader::activation::{DeviceId, TimeStamp};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use log::debug;

use crate::sim::core::Core;
use crate::utils::config::{ControllerSettings, DeviceType};

#[derive(Clone, Copy)]
pub(crate) struct Controller {
    pub(crate) id: DeviceId,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) aggregator: AggregatorType,
    pub(crate) status: DeviceState,
    pub(crate) device_type: DeviceType,
    pub(crate) device_class: u32,
    step: TimeStamp,
}

impl Controller {
    pub(crate) fn new(
        id: u64,
        timing_info: Timing,
        controller_settings: &ControllerSettings,
    ) -> Self {
        let aggregator: AggregatorType = match controller_settings.aggregator.name.as_str() {
            _ => AggregatorType::Basic(BasicAggregator {}),
        };

        Self {
            id,
            location: Real2D::default(),
            timing: timing_info,
            aggregator,
            status: DeviceState::Inactive,
            step: 0,
            device_type: DeviceType::Controller,
            device_class: controller_settings.device_class,
        }
    }

    fn update_geo_data(&mut self, core_state: &mut Core) {
        match core_state.device_field.position_cache.remove(&self.id) {
            Some(loc) => {
                self.location = loc;
                core_state
                    .device_field
                    .controller_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }
    }

    fn deactivate(&mut self, core_state: &mut Core) {
        self.status = DeviceState::Inactive;
        self.timing.increment_timing_index();
        core_state.devices_to_pop.controllers.push(self.id);

        let time_stamp = self.timing.pop_activation_time();
        if time_stamp > self.step {
            core_state
                .devices_to_add
                .controllers
                .push((self.id, time_stamp));
        }
    }
}

impl Agent for Controller {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        debug!("{} is active", self.id);
        self.status = DeviceState::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);

        // Initiate deactivation if it is time
        if self.step == self.timing.peek_deactivation_time() {
            self.deactivate(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        false
    }
}

impl Hash for Controller {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for Controller {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, loc: Real2D) {
        self.location = loc;
    }
}

impl fmt::Display for Controller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for Controller {}

impl PartialEq for Controller {
    fn eq(&self, other: &Controller) -> bool {
        self.id == other.id
    }
}
