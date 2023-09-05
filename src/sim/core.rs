use std::any::Any;

use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::sim::field::DeviceField;
use crate::sim::vanet::Vanet;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::{config, ds_config};
use crate::DISCRETIZATION;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field::Field;
use krabmaga::hashbrown::HashMap;
use krabmaga::{
    engine::{fields::field_2d::Field2D, location::Real2D, schedule::Schedule, state::State},
    rand::{self, Rng},
};
use log::info;

/// Expand the state definition according to your sim, for example by having a grid struct field
/// to store the agents' locations.
pub(crate) struct Core {
    pub(crate) config: config::Config,
    pub(crate) ds_config: ds_config::AllDataSources,
    pub(crate) step: u64,
    pub(crate) vehicles: HashMap<u64, Vehicle>,
    pub(crate) roadside_units: HashMap<u64, RoadsideUnit>,
    pub(crate) base_stations: HashMap<u64, BaseStation>,
    pub(crate) controllers: HashMap<u64, Controller>,
    pub(crate) device_field: DeviceField,
    pub(crate) vanet: Vanet,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct Timing {
    pub(crate) activation: [Option<u64>; ARRAY_SIZE],
    pub(crate) deactivation: [Option<u64>; ARRAY_SIZE],
}

impl Core {
    pub(crate) fn new(
        config: config::Config,
        ds_config: ds_config::AllDataSources,
        vehicles: HashMap<u64, Vehicle>,
        roadside_units: HashMap<u64, RoadsideUnit>,
        base_stations: HashMap<u64, BaseStation>,
        controllers: HashMap<u64, Controller>,
        device_field: DeviceField,
        vanet: Vanet,
    ) -> Self {
        Self {
            config,
            ds_config,
            step: 0,
            vehicles,
            roadside_units,
            base_stations,
            controllers,
            device_field,
            vanet,
        }
    }

    pub(crate) fn get_duration(&self) -> u64 {
        return self.config.simulation_settings.sim_duration;
    }
}

impl State for Core {
    /// Put the code that should be executed to initialize simulation:
    /// Agent creation and schedule set-up
    fn init(&mut self, schedule: &mut Schedule) {
        info!("Initializing simulation...");
        self.device_field.init();
        self.vanet.init();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_state_mut(&mut self) -> &mut dyn State {
        self
    }

    fn as_state(&self) -> &dyn State {
        self
    }

    /// Put the code that should be executed to reset simulation state
    fn reset(&mut self) {}

    /// Put the code that should be executed for each state update here. The state is updated once for each
    /// schedule step.
    fn update(&mut self, _step: u64) {
        info!("Updating state at step {}", self.step);
        self.device_field.update();
    }

    fn after_step(&mut self, _schedule: &mut Schedule) {
        self.step += 1
    }
}
