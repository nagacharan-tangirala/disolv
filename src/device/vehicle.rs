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
    pub(crate) vehicle_info: VehicleInfo,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VehiclePayload {
    pub(crate) id: u32,
    pub(crate) vehicle_info: VehicleInfo,
    pub(crate) generated_data_size: HashMap<SensorType, f32>,
    pub(crate) types_with_counts: HashMap<SensorType, u16>,
    pub(crate) preferred_targets: HashMap<SensorType, DataTargetType>,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct VehicleInfo {
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
            vehicle_info: VehicleInfo::default(),
            composer,
            simplifier,
        }
    }

    pub(crate) fn get_vehicle_info(&self) -> VehicleInfo {
        self.vehicle_info
    }
}

impl Agent for Vehicle {
    /// Put the code that should happen for each step, for each agent here.
    fn step(&mut self, state: &mut dyn State) {
        let state = state.as_any().downcast_ref::<Core>().unwrap();
        let mut rng = rand::thread_rng();

        self.location = Real2D { x: 0.0, y: 0.0 };

        // state
        //     .vehicle_field
        //     .set_object_location(*self, Real2D { x: 0.0, y: 0.0 });
    }

    /// Put the code that decides if an agent should be removed or not
    /// for example in simulation where agents can die
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
