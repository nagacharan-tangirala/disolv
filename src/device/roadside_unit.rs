use core::fmt;
use std::hash::{Hash, Hasher};

use crate::data::data_io::DeviceId;
use crate::device::device_state::{DeviceState, Timing};
use crate::models::composer::{BasicComposer, ComposerType, RandomComposer};
use crate::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::{toroidal_transform, Location2D};
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::hashbrown::HashMap;
use krabmaga::rand;
use krabmaga::rand::Rng;
use log::debug;

use crate::sim::core::Core;
use crate::utils::config::RSUSettings;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, DataTargetType, SensorType};

/// The most basic agent should implement Clone, Copy and Agent to be able to be inserted in a Schedule.
#[derive(Debug, Clone, Copy)]
pub struct RoadsideUnit {
    pub(crate) id: DeviceId,
    storage: f32,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) vehicle_info: SensorInfo,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
    pub(crate) status: DeviceState,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RSUPayload {
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

impl RoadsideUnit {
    pub(crate) fn new(id: u64, timing_info: Timing, rsu_settings: &RSUSettings) -> Self {
        let data_sources: [Option<DataSourceSettings>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        let composer: ComposerType = match rsu_settings.composer.name.as_str() {
            "random" => ComposerType::Random(RandomComposer {
                data_sources: data_sources.clone(),
            }),
            _ => ComposerType::Basic(BasicComposer {
                data_sources: data_sources.clone(),
            }),
        };
        let simplifier: SimplifierType = match rsu_settings.composer.name.as_str() {
            "random" => SimplifierType::Random(RandomSimplifier {}),
            _ => SimplifierType::Basic(BasicSimplifier {}),
        };

        Self {
            id,
            storage: rsu_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            vehicle_info: SensorInfo::default(),
            composer,
            simplifier,
            status: DeviceState::Inactive,
        }
    }
}

impl Agent for RoadsideUnit {
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
                    .rsu_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }

        // If it is time to deactivate, schedule deactivation
        if step == self.timing.peek_deactivation_time() {
            self.status = DeviceState::Inactive;
            self.timing.increment_timing_index();
            core_state.devices_to_pop.roadside_units.push(self.id);

            let time_stamp = self.timing.pop_activation_time();
            if time_stamp > step {
                core_state
                    .devices_to_add
                    .roadside_units
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

impl Hash for RoadsideUnit {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for RoadsideUnit {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, new_location: Real2D) {
        self.location = new_location;
    }
}

impl fmt::Display for RoadsideUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for RoadsideUnit {}

impl PartialEq for RoadsideUnit {
    fn eq(&self, other: &RoadsideUnit) -> bool {
        self.id == other.id
    }
}
