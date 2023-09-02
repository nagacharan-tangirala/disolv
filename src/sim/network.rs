use std::any::Any;

use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::sim::field::DeviceField;
use crate::sim::vanet::Vanet;
use crate::utils::config;
use crate::DISCRETIZATION;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field::Field;
use krabmaga::hashbrown::HashMap;
use krabmaga::{
    engine::{fields::field_2d::Field2D, location::Real2D, schedule::Schedule, state::State},
    rand::{self, Rng},
};

/// Expand the state definition according to your sim, for example by having a grid struct field
/// to store the agents' locations.
pub(crate) struct Network {
    pub(crate) config: config::Config,
    pub(crate) step: u64,
    pub(crate) vehicles: HashMap<i32, Vehicle>,
    pub(crate) roadside_units: HashMap<i32, RoadsideUnit>,
    pub(crate) base_stations: HashMap<i32, BaseStation>,
    pub(crate) controllers: HashMap<i32, Controller>,
    pub(crate) device_field: DeviceField,
    pub(crate) vanet: Vanet,
}

impl Network {
    pub(crate) fn new(
        config: config::Config,
        vehicles: HashMap<i32, Vehicle>,
        roadside_units: HashMap<i32, RoadsideUnit>,
        base_stations: HashMap<i32, BaseStation>,
        controllers: HashMap<i32, Controller>,
        device_field: DeviceField,
        vanet: Vanet,
    ) -> Self {
        Network {
            config,
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

impl State for Network {
    /// Put the code that should be executed to initialize simulation:
    /// Agent creation and schedule set-up
    fn init(&mut self, schedule: &mut Schedule) {
        info!("Initializing simulation.");
        self.device_field.init();
        //self.vanet.init();
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
