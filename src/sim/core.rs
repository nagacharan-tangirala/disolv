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

    fn deactivate_devices(&mut self) {
        for vehicle_id in self.devices_to_pop.vehicles.iter() {
            if let Some(vehicle) = self.vehicles.get_mut(vehicle_id) {
                vehicle.status = DeviceState::Inactive;
            } else {
                panic!("Vehicle {} not found", vehicle_id);
            }
        }

        for rsu_id in self.devices_to_pop.roadside_units.iter() {
            if let Some(rsu) = self.roadside_units.get_mut(rsu_id) {
                rsu.status = DeviceState::Inactive;
            } else {
                panic!("RSU {} not found", rsu_id);
            }
        }

        for bs_id in self.devices_to_pop.base_stations.iter() {
            if let Some(bs) = self.base_stations.get_mut(bs_id) {
                bs.status = DeviceState::Inactive;
            } else {
                panic!("Base station {} not found", bs_id);
            }
        }

        for controller_id in self.devices_to_pop.controllers.iter() {
            if let Some(controller) = self.controllers.get_mut(controller_id) {
                controller.status = DeviceState::Inactive;
            } else {
                panic!("Controller {} not found", controller_id);
            }
        }
    }
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

    fn reset(&mut self) {}

    fn update(&mut self, step: u64) {
        info!("Updating state at step {}", self.step);
        self.device_field.update();
        self.step = step;
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        info!("Before step {}", self.step);
        self.device_field.before_step(self.step);
        for vehicle_ts in self.devices_to_add.vehicles.iter() {
            if let Some(vehicle) = self.vehicles.get_mut(&vehicle_ts.0) {
                if !schedule.schedule_repeating(Box::new(*vehicle), vehicle_ts.1 as f32, 0) {
                    error!("Could not schedule vehicle {} ", vehicle.id);
                    panic!("Could not schedule vehicle {} ", vehicle.id);
                }
            }
        }

        for rsu_ts in self.devices_to_add.roadside_units.iter_mut() {
            if let Some(rsu) = self.roadside_units.get_mut(&rsu_ts.0) {
                if !schedule.schedule_repeating(Box::new(*rsu), rsu_ts.1 as f32, 1) {
                    error!("Could not schedule vehicle {} ", rsu.id);
                    panic!("Could not schedule vehicle {} ", rsu_ts.0);
                }
            }
        }

        for base_station_ts in self.devices_to_add.base_stations.iter() {
            if let Some(base_station) = self.base_stations.get_mut(&base_station_ts.0) {
                if !schedule.schedule_repeating(
                    Box::new(*base_station),
                    base_station_ts.1 as f32,
                    2,
                ) {
                    error!("Could not schedule vehicle {} ", base_station.id);
                    panic!("Could not schedule vehicle {} ", base_station_ts.0);
                }
            }
        }

        for controller_ts in self.devices_to_add.controllers.iter() {
            if let Some(controller) = self.controllers.get_mut(&controller_ts.0) {
                if !schedule.schedule_repeating(Box::new(*controller), controller_ts.1 as f32, 3) {
                    error!("Could not schedule vehicle {} ", controller.id);
                    panic!("Could not schedule vehicle {} ", controller_ts.0);
                }
            }
        }

        if self.step > 0 && self.step % self.config.simulation_settings.sim_streaming_step == 0 {
            self.vanet.refresh_links_data(self.step);
            self.device_field.refresh_position_data(self.step);
        }

        self.devices_to_add.clear();
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        info!("After step {}", self.step);
        self.deactivate_devices();
        self.devices_to_pop.clear();

        let agents = schedule.get_all_events();
        let mut num_vehicles: f32 = 0.;
        let mut num_bs: f32 = 0.;

        for n in agents {
            if let Some(v) = n.downcast_ref::<Vehicle>() {
                if v.status == DeviceState::Active {
                    num_vehicles += 1.;
                }
            }
            if let Some(_w) = n.downcast_ref::<BaseStation>() {
                num_bs += 1.;
            }
        }

        plot!(
            String::from("Agents"),
            String::from("Base Stations"),
            self.step as f64,
            num_bs as f64
        );

        plot!(
            String::from("Agents"),
            String::from("Vehicles"),
            self.step as f64,
            num_vehicles as f64
        );
    }
}
