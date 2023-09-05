use std::any::Any;

use crate::data::data_io::TimeStamp;
use crate::device::base_station::BaseStation;
use crate::device::controller::Controller;
use crate::device::roadside_unit::RoadsideUnit;
use crate::device::vehicle::Vehicle;
use crate::sim::field::DeviceField;
use crate::sim::vanet::Vanet;
use crate::utils::{config, ds_config};
use krabmaga::engine::fields::field::Field;
use krabmaga::hashbrown::HashMap;
use krabmaga::{
    engine::{fields::field_2d::Field2D, location::Real2D, schedule::Schedule, state::State},
    rand::{self, Rng},
};
use log::{debug, info};

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
    pub(crate) devices_to_add: DevicesToAdd,
    pub(crate) devices_to_pop: DevicesToRemove,
}

#[derive(Clone, Default)]
pub(crate) struct DevicesToAdd {
    pub(crate) vehicles: Vec<(Vehicle, TimeStamp)>,
    pub(crate) roadside_units: Vec<(RoadsideUnit, TimeStamp)>,
    pub(crate) base_stations: Vec<(BaseStation, TimeStamp)>,
    pub(crate) controllers: Vec<(Controller, TimeStamp)>,
}

#[derive(Clone, Default)]
pub(crate) struct DevicesToRemove {
    pub(crate) vehicles: Vec<Vehicle>,
    pub(crate) roadside_units: Vec<RoadsideUnit>,
    pub(crate) base_stations: Vec<BaseStation>,
    pub(crate) controllers: Vec<Controller>,
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
            devices_to_add: DevicesToAdd::default(),
            devices_to_pop: DevicesToRemove::default(),
        }
    }

    pub(crate) fn get_duration(&self) -> u64 {
        return self.config.simulation_settings.sim_duration;
    }

    pub(crate) fn schedule_activations(&mut self) {}
}

impl DevicesToAdd {
    pub(crate) fn clear(&mut self) {
        self.vehicles.clear();
        self.roadside_units.clear();
        self.base_stations.clear();
        self.controllers.clear();
    }
}

impl DevicesToRemove {
    pub(crate) fn clear(&mut self) {
        self.vehicles.clear();
        self.roadside_units.clear();
        self.base_stations.clear();
        self.controllers.clear();
    }
}

impl State for Core {
    fn init(&mut self, schedule: &mut Schedule) {
        info!("Initializing simulation...");
        self.device_field.init();
        self.vanet.init();
        info!("Scheduling activation of the devices");
        for (_, vehicle) in self.vehicles.iter_mut() {
            debug!("Activating vehicle {}", vehicle.id);
            let time_stamp = vehicle.timing.pop_activation_time();
            self.devices_to_add.vehicles.push((*vehicle, time_stamp));
        }
        for (_, roadside_unit) in self.roadside_units.iter_mut() {
            debug!("Activating RSU {}", roadside_unit.id);
            let time_stamp = roadside_unit.timing.pop_activation_time();
            self.devices_to_add
                .roadside_units
                .push((*roadside_unit, time_stamp));
        }
        for (_, base_station) in self.base_stations.iter_mut() {
            debug!("Activating base_station {}", base_station.id);
            let time_stamp = base_station.timing.pop_activation_time();
            self.devices_to_add
                .base_stations
                .push((*base_station, time_stamp));
        }
        for (_, controller) in self.controllers.iter_mut() {
            debug!("Activating controller {}", controller.id);
            let time_stamp = controller.timing.pop_activation_time();
            self.devices_to_add
                .controllers
                .push((*controller, time_stamp));
        }
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

    fn before_step(&mut self, schedule: &mut Schedule) {
        info!("Before step {}", self.step);
        for vehicle in self.devices_to_add.vehicles.iter() {
            schedule.schedule_repeating(Box::new(vehicle.0), vehicle.1 as f32, 0);
        }
        for roadside_unit in self.devices_to_add.roadside_units.iter() {
            schedule.schedule_repeating(Box::new(roadside_unit.0), roadside_unit.1 as f32, 1);
        }
        for base_station in self.devices_to_add.base_stations.iter() {
            schedule.schedule_repeating(Box::new(base_station.0), base_station.1 as f32, 2);
        }
        for controller in self.devices_to_add.controllers.iter() {
            schedule.schedule_repeating(Box::new(controller.0), controller.1 as f32, 3);
        }
        self.devices_to_add.clear();
    }

    fn update(&mut self, step: u64) {
        info!("Updating state at step {}", self.step);

        self.device_field.update();
        self.step = step;
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        info!("After step {}", self.step);
        for vehicle in self.devices_to_pop.vehicles.iter() {
            schedule.dequeue(Box::new(*vehicle), vehicle.id as u32);
        }
        for roadside_unit in self.devices_to_pop.roadside_units.iter() {
            schedule.dequeue(Box::new(*roadside_unit), roadside_unit.id as u32);
        }
        for base_station in self.devices_to_pop.base_stations.iter() {
            schedule.dequeue(Box::new(*base_station), base_station.id as u32);
        }
        for controller in self.devices_to_pop.controllers.iter() {
            schedule.dequeue(Box::new(*controller), controller.id as u32);
        }
        self.devices_to_pop.clear();
        self.step += 1;
    }
}
