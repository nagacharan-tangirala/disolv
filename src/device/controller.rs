use core::fmt;
use std::hash::{Hash, Hasher};

use crate::data::data_io::DeviceId;
use crate::device::device_state::{DeviceState, Timing};
use crate::models::aggregator::{AggregatorType, BasicAggregator};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;

use crate::sim::core::Core;
use crate::utils::config::ControllerSettings;

/// The most basic agent should implement Clone, Copy and Agent to be able to be inserted in a Schedule.
#[derive(Clone, Copy)]
pub(crate) struct Controller {
    pub(crate) id: DeviceId,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) aggregator: AggregatorType,
    pub(crate) status: DeviceState,
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
        }
    }
}

impl Agent for Controller {
    /// Put the code that should happen for each step, for each agent here.
    fn step(&mut self, state: &mut dyn State) {
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        let step = core_state.step;

        // If we are scheduled, we are active
        self.status = DeviceState::Active;

        match core_state.device_field.position_cache.get(&self.id) {
            Some(loc) => {
                self.location = *loc;
                core_state
                    .device_field
                    .controller_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }

        // If it is time to deactivate, schedule deactivation
        if step == self.timing.peek_deactivation_time() {
            self.status = DeviceState::Inactive;
            self.timing.increment_timing_index();
            core_state.devices_to_pop.controllers.push(self.id);

            let time_stamp = self.timing.pop_activation_time();
            if time_stamp > step {
                core_state
                    .devices_to_add
                    .controllers
                    .push((self.id, time_stamp));
            }
        }
    }

    /// Put the code that decides if an agent should be removed or not
    /// for example in simulation where agents can die
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
