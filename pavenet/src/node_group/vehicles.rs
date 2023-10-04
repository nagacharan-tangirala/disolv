use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use log::error;
use pavenet_config::config::types::{NodeId, TimeStamp};
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use pavenet_core::core::core::Core;
use pavenet_core::node::group::NodeGroup;
use pavenet_models::node_group::space::Space;
use pavenet_models::node_group::vanet::Vanet;

pub struct Vehicles {
    vehicles: Vec<Vehicle>,
    by_class: HashMap<u32, NodeId>,
    by_hierarchy: HashMap<i32, NodeId>,
    pub(crate) to_add: Vec<(NodeId, TimeStamp)>,
    pub(crate) to_pop: Vec<NodeId>,
    pub(crate) space: Space,
    pub vanet: Vanet,
}

impl Vehicles {
    pub fn new(space: Space) -> Self {
        Self {
            space,
            ..Default::default()
        }
    }
}

impl NodeGroup for Vehicles {
    fn init(&mut self, schedule: &mut Schedule) {
        for vehicle in self.vehicles.iter() {
            self.by_class.insert(vehicle.device_class, vehicle.id);
            self.by_hierarchy.insert(vehicle.hierarchy, vehicle.id);
        }
        self.space.init(TimeStamp::from(schedule.step));
    }

    fn before_step(&mut self, step: TimeStamp) {
        self.space.update_map_cache(step);
    }

    fn update(&mut self, step: TimeStamp) {
        todo!()
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        todo!()
    }

    fn streaming_step(&mut self, step: TimeStamp) {
        self.space.stream_map_states(step);
    }
}

impl NodeCollection for Vehicles {
    fn streaming_step(&mut self, step: TimeStamp) {}

    fn add_all(&mut self) {
        for vehicle in self.vehicles.iter_mut() {
            let time_stamp = vehicle.models.power_schedule.pop_time_to_on();
            self.to_add.push((vehicle.id, time_stamp));
        }
    }

    fn power_off(&mut self, schedule: &mut Schedule) {
        for vehicle_id in self.to_pop.into_iter() {
            if let Some(vehicle) = self.vehicles.get_mut(vehicle_id) {
                vehicle.power_state = PowerState::Off;
                schedule.dequeue(Box::new(*vehicle), vehicle.id.into());
            } else {
                panic!("Vehicle {} not found", vehicle_id);
            }
        }
    }

    fn schedule_power_on(&mut self, schedule: &mut Schedule) {
        for vehicle_ts in self.to_add.into_iter() {
            let vehicle: Option<&mut Vehicle> = self.vehicles.get_mut(&vehicle_ts.0);
            match vehicle {
                Some(vehicle) => schedule.schedule_repeating(
                    Box::new(*vehicle),
                    vehicle.device_info.id.into(),
                    vehicle_ts.1 as f32,
                    vehicle.transmitter.hierarchy.as_i32(),
                ),
                None => {
                    error!("Could not find vehicle {}", vehicle_ts.0);
                    panic!("Could not find vehicle {}", vehicle_ts.0);
                }
            }
        }
    }
}
