use crate::core::core::Core;
use crate::core::node::Node;
use crate::devices::model::{DeviceModel, ModelBuilder};
use crate::node::info::NodeInfo;
use crate::node::map::MapState;
use crate::node::power::{PowerControl, PowerSchedule, PowerState};
use crate::node::receive::{DataReceiver, Recipient};
use crate::node::transmit::{DataTransmitter, Payload, SensorData, Transmitter};
use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use log::debug;
use pavenet_config::config::base::{DeviceSettings, NodeType};
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug)]
pub struct Vehicle {
    pub device_info: NodeInfo,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub receiver: DataReceiver,
    pub models: DeviceModel,
    step: TimeStamp,
}

impl Transmitter for Vehicle {
    fn collect_from_sensors(&self) -> SensorData {
        match self.models.composer {}
    }

    fn build_payload(&self) -> Payload {
        todo!()
    }

    fn transmit(&self, data: &SensorData) {
        self.transmit(data)
    }
}

impl Vehicle {
    pub fn new(
        id: NodeId,
        power_schedule: PowerSchedule,
        vehicle_settings: &DeviceSettings,
    ) -> Self {
        let models: DeviceModel = ModelBuilder::new()
            .with_composer(&vehicle_settings.composer)
            .with_simplifier(&vehicle_settings.simplifier)
            .with_linker(vehicle_settings.linker.clone())
            .with_power_schedule(power_schedule)
            .build();

        Self {
            id,
            models,
            device_class: vehicle_settings.device_class,
            device_type: NodeType::Vehicle,
            ..Default::default()
        }
    }

    fn power_off(&mut self, core: &mut Core) {
        self.device_info.power_state = PowerState::Off;
        self.models.power_schedule.pop_time_to_off();
        core.vehicles.to_pop.push(self.device_info.id);

        let time_stamp = self.models.power_schedule.pop_time_to_on();
        if time_stamp > self.step {
            core.vehicles.to_add.push((self.device_info.id, time_stamp));
        }
    }

    pub fn setup_episode(&mut self, episode: EpisodeInfo) {
        match episode.episode_type {
            EpisodeType::Temporary => {
                let duration = match episode.duration {
                    Some(duration) => duration,
                    None => panic!("Duration must be specified for temporary episodes."),
                };
                self.update_models(&episode, self.step + duration);
            }
            EpisodeType::Persistent => self.update_models(&episode, 0),
        }
    }

    fn collect_data(&mut self, core_state: &mut Core) {
        let mut response_from_bs = core_state.vanet.downlink.bs2v_responses.remove(&self.id);
        self.veh_data_stats.incoming_rsu_data_size = 0.0;
        self.veh_data_stats.incoming_v2v_data_size = 0.0;
    }

    #[allow(dead_code)]
    pub fn as_agent(self) -> Box<dyn Agent> {
        Box::new(self)
    }
}

impl PowerControl for Vehicle {
    fn power_state(&self) -> PowerState {
        todo!()
    }

    fn set_power_state(&mut self, power_state: PowerState) {
        todo!()
    }
}

impl Recipient for Vehicle {
    fn receive(&self, data: &mut Vec<Payload>) {
        todo!()
    }
}

impl Node for Vehicle {
    fn as_agent(self) -> Box<dyn Agent> {
        Box::new(self)
    }

    fn node_info(&self) -> NodeInfo {
        todo!()
    }

    fn power_schedule(&self) -> PowerSchedule {
        todo!()
    }

    fn set_power(&mut self, power_state: PowerState) {
        self.power_state = power_state;
    }

    fn update_map_state(&mut self, core_state: &mut Core) {
        match core_state
            .vehicles
            .space
            .map_cache
            .remove(&self.device_info.id)
        {
            Some(map_state) => {
                self.device_info.map_state = map_state;
                core_state
                    .vehicles
                    .space
                    .add_device(self.device_info.id, map_state.pos);
            }
            None => {}
        }
    }

    fn transmit_data(&mut self, state: &mut Core) {
        todo!()
    }

    fn receive_data(&mut self, state: &mut Core) {
        todo!()
    }

    fn collect_stats(&mut self, state: &mut Core) {
        todo!()
    }
}

impl Agent for Vehicle {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is ON", self.id);
        self.device_info.set_power_state(PowerState::On);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;
        self.update_map_state(core_state);
        self.transmit_data(core_state);

        // Initiate deactivation if it is time
        if self.step == self.models.power_schedule.peek_deactivation_time() {
            self.power_off(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        self.device_info.is_off()
    }

    fn after_step(&mut self, state: &mut dyn State) -> () {
        debug!("After step {}", self.step);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.receive_data(core_state);
        self.collect_data(core_state);
    }
}

impl Hash for Vehicle {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.device_info.id.hash(state);
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.device_info.id)
    }
}

impl Eq for Vehicle {}

impl PartialEq for Vehicle {
    fn eq(&self, other: &Vehicle) -> bool {
        self.device_info.id == other.device_info.id
    }
}
