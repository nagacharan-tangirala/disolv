use crate::node::model::DeviceModel;
use pavenet_core::structs::{MapState, NodeInfo};
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_engine::engine::core::Core;
use pavenet_engine::node::node::Node;
use pavenet_engine::node::power::{PowerSchedule, PowerState};
use pavenet_engine::node::receive::Recipient;
use pavenet_engine::node::transmit::{Transferable, Transmitter};

#[derive(Clone, Copy, Debug)]
pub struct Device {
    pub node_info: NodeInfo,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub models: DeviceModel,
    step: TimeStamp,
}

impl Device {
    pub fn new(id: NodeId, power_schedule: PowerSchedule, vehicle_settings: &NodeSettings) -> Self {
        let models: DeviceModel = ModelBuilder::new()
            .with_composer(&vehicle_settings.composer)
            .with_simplifier(&vehicle_settings.simplifier)
            .with_linker(vehicle_settings.linker.clone())
            .with_power_schedule(power_schedule)
            .build();

        Self {
            id,
            models,
            device_class: vehicle_settings.node_class,
            device_type: NodeType::Vehicle,
            ..Default::default()
        }
    }

    fn power_off(&mut self, core: &mut Core) {
        self.node_info.power_state = PowerState::Off;
        self.models.power_schedule.pop_time_to_off();
        core.vehicles.to_pop.push(self.node_info.id);

        let time_stamp = self.models.power_schedule.pop_time_to_on();
        if time_stamp > self.step {
            core.vehicles.to_add.push((self.node_info.id, time_stamp));
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
}

impl Transmitter for Device {
    fn generate_data(&mut self, core: &mut Core) {
        todo!()
    }

    fn transmit(&mut self, payload: Box<dyn Transferable>) {
        todo!()
    }
}

impl Recipient for Device {
    fn receive(&mut self, payloads: &mut Vec<Box<dyn Transferable>>) {
        todo!()
    }

    fn report_stats(&mut self, core: &mut Core) {
        todo!()
    }
}

impl Node for Device {
    fn power_state(&self) -> PowerState {
        self.power_state
    }

    fn node_info(&self) -> NodeInfo {
        self.node_info
    }

    fn set_power_state(&mut self, power_state: PowerState) {
        self.power_state = power_state;
    }

    fn step(&mut self, core: &mut Core) {
        self.step = core.step;
        self.update_map_state(core);
        self.generate_data(core);
    }

    fn after_step(&mut self, core: &mut Core) {
        self.receive_data(core);
        self.report_stats(core);
    }

    fn node_order(&self) -> i32 {
        todo!()
    }
}
