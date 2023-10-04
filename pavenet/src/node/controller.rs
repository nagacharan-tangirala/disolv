use core::fmt;
use std::hash::{Hash, Hasher};

use pavenet_device::device::common::{Status, Timing};
use crate::models::composer::controller::{BasicControllerComposer, ControllerComposerType};
use pavenet::models::composer::{BasicComposer, ComposerType, StatusComposer};
use pavenet::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use log::debug;

use pavenet_core::::config::{ControllerSettings, DeviceSettings, DeviceType};
use crate::sim::core::Core;

#[derive(Clone, Copy)]
pub struct Controller {
    pub id: DeviceId,
    pub location: Real2D,
    pub timing: Timing,
    pub composer: ControllerComposerType,
    pub status: Status,
    pub device_type: DeviceType,
    pub device_class: u32,
    step: TimeStamp,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct ControllerInfo {
    pub location: Real2D,
    pub temperature: f32,
    pub storage: f32,
}

impl Controller {
    pub fn new(id: u64, timing_info: Timing, controller_settings: &DeviceSettings) -> Self {
        let composer: Option<ComposerType> = match controller_settings.composer {
            Some(ref composer_settings) => match composer_settings.name.as_str() {
                "basic" => Some(ComposerType::Basic(BasicComposer::new(composer_settings))),
                "status" => Some(ComposerType::Status(StatusComposer::new(composer_settings))),
                _ => panic!("Unknown composer type"),
            },
            None => None,
        };

        let simplifier: Option<SimplifierType> = match controller_settings.simplifier {
            Some(ref simplifier_settings) => match simplifier_settings.name.as_str() {
                "basic" => Some(SimplifierType::Basic(BasicSimplifier::new(
                    simplifier_settings,
                ))),
                "random" => Some(SimplifierType::Random(RandomSimplifier::new(
                    simplifier_settings,
                ))),
                _ => panic!("Unknown simplifier type"),
            },
            None => None,
        };

        Self {
            id,
            timing: timing_info,
            device_class: controller_settings.device_class,
            composer,
            step: TimeStamp::default(),
            location: Real2D::default(),
            status: Status::Inactive,
            device_type: DeviceType::Controller,
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
        self.status = Status::Inactive;
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

    fn read_incoming_data(&mut self, core_state: &mut Core) {
        let bs_data = match core_state.vanet.uplink.bs2c_data.remove(&self.id) {
            Some(bs_data) => bs_data,
            None => return,
        };

        // Send responses to base stations.

        let controller_payload = match self.composer {
            ControllerComposerType::Basic(ref composer) => composer.compose_c2c_payload(bs_data),
        };
    }
}

impl Agent for Controller {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        debug!("{} is active", self.id);
        self.status = Status::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);
        self.read_incoming_data(core_state);

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
