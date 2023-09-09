use crate::device::device_state::{DeviceState, Timing};
use crate::models::composer::{
    BasicComposer, ComposerType, DevicePayload, RandomComposer, SensorData,
};
use crate::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::schedule::ScheduleOptions;
use krabmaga::engine::state::State;
use log::debug;
use std::hash::{Hash, Hasher};

use crate::sim::core::Core;
use crate::sim::vanet::Link;
use crate::utils::config::VehicleSettings;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, DataTargetType};

#[derive(Clone, Debug, Copy)]
pub(crate) struct Vehicle {
    pub(crate) id: DeviceId,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) sensor_data: SensorData,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
    pub(crate) status: DeviceState,
    storage: f32,
    step: TimeStamp,
}

impl Vehicle {
    pub(crate) fn new(
        id: u64,
        timing_info: Timing,
        vehicle_settings: &VehicleSettings,
        data_sources: [Option<DataSourceSettings>; ARRAY_SIZE],
    ) -> Self {
        let composer: ComposerType = match vehicle_settings.composer.name.as_str() {
            "random" => ComposerType::Random(RandomComposer::new(data_sources.clone())),
            _ => ComposerType::Basic(BasicComposer::new(data_sources.clone())),
        };
        let simplifier: SimplifierType = match vehicle_settings.simplifier.name.as_str() {
            "random" => SimplifierType::Random(RandomSimplifier {}),
            _ => SimplifierType::Basic(BasicSimplifier::new(vehicle_settings.simplifier.clone())),
        };

        Self {
            id,
            storage: vehicle_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            sensor_data: SensorData::default(),
            composer,
            simplifier,
            status: DeviceState::Inactive,
            step: 0,
        }
    }

    fn update_geo_data(&mut self, core_state: &mut Core) {
        match core_state.device_field.position_cache.remove(&self.id) {
            Some(loc) => {
                self.location = loc;
                core_state
                    .device_field
                    .vehicle_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }
        self.sensor_data.speed = match core_state.device_field.velocity_cache.remove(&self.id) {
            Some(speed) => speed,
            None => 0.0,
        };
    }

    pub(crate) fn deactivate(&mut self, core_state: &mut Core) {
        self.status = DeviceState::Inactive;
        self.timing.increment_timing_index();
        core_state.devices_to_pop.vehicles.push(self.id);

        let time_stamp = self.timing.pop_activation_time();
        if time_stamp > self.step {
            core_state
                .devices_to_add
                .vehicles
                .push((self.id, time_stamp));
        }
    }

    pub(crate) fn transfer_data_to_vehicles(&mut self, core_state: &mut Core) {
        let mut v2v_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DataTargetType::Vehicle),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };

        v2v_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2v_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2v_payload),
        };

        v2v_payload.id = self.id;
        v2v_payload.sensor_data = self.sensor_data;
    }

    pub(crate) fn transfer_data_to_bs(&mut self, core_state: &mut Core) {
        let mut v2bs_payload = match self.composer {
            ComposerType::Basic(ref composer) => {
                composer.compose_payload(DataTargetType::BaseStation)
            }
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        v2bs_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2bs_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2bs_payload),
        };
        v2bs_payload.id = self.id;
        v2bs_payload.sensor_data = self.sensor_data;
    }

    pub(crate) fn transfer_data_to_rsu(&mut self, core_state: &mut Core) {
        let mut v2rsu_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DataTargetType::RSU),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        v2rsu_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2rsu_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2rsu_payload),
        };
        v2rsu_payload.id = self.id;
        v2rsu_payload.sensor_data = self.sensor_data;
    }

    pub fn as_agent(self) -> Box<dyn Agent> {
        Box::new(self)
    }
}

impl Agent for Vehicle {
    fn step(&mut self, state: &mut dyn State) {
        self.status = DeviceState::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);
        self.transfer_data_to_rsu(core_state);
        self.transfer_data_to_bs(core_state);
        self.transfer_data_to_vehicles(core_state);

        // Initiate deactivation if it is time
        if self.step == self.timing.peek_deactivation_time() {
            self.deactivate(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        self.status == DeviceState::Inactive
    }
}

impl Hash for Vehicle {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for Vehicle {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, loc: Real2D) {
        self.location = loc;
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for Vehicle {}

impl PartialEq for Vehicle {
    fn eq(&self, other: &Vehicle) -> bool {
        self.id == other.id
    }
}
