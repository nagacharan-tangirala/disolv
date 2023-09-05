use crate::data::data_io::TimeStamp;
use crate::device::device_state::{DeviceState, Timing};
use crate::models::composer::{BasicComposer, ComposerType, RandomComposer};
use crate::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::hashbrown::HashMap;
use krabmaga::rand;
use log::debug;
use std::hash::{Hash, Hasher};

use crate::sim::core::Core;
use crate::utils::config::VehicleSettings;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, DataTargetType, SensorType};

/// The most basic agent should implement Clone, Copy and Agent to be able to be inserted in a Schedule.
#[derive(Clone, Debug, Copy)]
pub(crate) struct Vehicle {
    pub(crate) id: u64,
    storage: f32,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) sensor_info: SensorInfo,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
    status: DeviceState,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VehiclePayload {
    pub(crate) id: u32,
    pub(crate) vehicle_info: SensorInfo,
    pub(crate) generated_data_size: HashMap<SensorType, f32>,
    pub(crate) types_with_counts: HashMap<SensorType, u16>,
    pub(crate) preferred_targets: HashMap<SensorType, DataTargetType>,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct SensorInfo {
    pub(crate) location: Real2D,
    pub(crate) speed: f32,
    pub(crate) temperature: f32,
    pub(crate) env_temperature: f32,
}

impl Vehicle {
    pub(crate) fn new(id: u64, timing_info: Timing, vehicle_settings: &VehicleSettings) -> Self {
        let data_sources: [Option<DataSourceSettings>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        let composer: ComposerType = match vehicle_settings.composer.name.as_str() {
            "random" => ComposerType::Random(RandomComposer {
                data_sources: data_sources.clone(),
            }),
            _ => ComposerType::Basic(BasicComposer {
                data_sources: data_sources.clone(),
            }),
        };
        let simplifier: SimplifierType = match vehicle_settings.composer.name.as_str() {
            "random" => SimplifierType::Random(RandomSimplifier {}),
            _ => SimplifierType::Basic(BasicSimplifier {}),
        };

        Self {
            id,
            storage: vehicle_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            sensor_info: SensorInfo::default(),
            composer,
            simplifier,
            status: DeviceState::Inactive,
        }
    }

    pub(crate) fn schedule_activation(&mut self, sim_core: &mut Core) {
        let step = sim_core.step;
        let time_stamp = self.timing.pop_activation_time();
        if time_stamp >= step {
            sim_core.devices_to_add.vehicles.push((*self, time_stamp));
        }
    }

    pub(crate) fn schedule_deactivation(&mut self, sim_core: &mut Core) {
        let step = sim_core.step;
    }

    pub(crate) fn get_vehicle_info(&self) -> SensorInfo {
        self.sensor_info
    }
}

impl Agent for Vehicle {
    fn step(&mut self, state: &mut dyn State) {
        debug!("Vehicle {} step", self.id);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        let step = core_state.step;
        // If we are scheduled, we are active
        self.status = DeviceState::Active;
        // Do data transfers.

        // If it is time to deactivate, schedule deactivation
        if step == self.timing.peek_deactivation_time() {
            self.status = DeviceState::Inactive;
            // Add to devices to pop at this time step.
            let time_stamp = self.timing.pop_deactivation_time();
            core_state.devices_to_pop.vehicles.push(*self);

            // Add to devices to add at the next activation time step.
            let time_stamp = self.timing.pop_activation_time();
            core_state.devices_to_add.vehicles.push((*self, time_stamp));
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        false
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
