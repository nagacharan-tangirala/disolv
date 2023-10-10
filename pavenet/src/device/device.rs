use crate::device::model::{DeviceModel, ModelBuilder};
use crate::pool::episode::NodeChanges;
use pavenet_core::structs::{MapState, NodeInfo};
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_engine::engine::core::Core;
use pavenet_engine::node::node::Node;
use pavenet_engine::node::power::{PowerSchedule, PowerState};
use pavenet_models::node::payload::Payload;
use pavenet_models::node::traits::Transmitter;

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

    fn collect_data(&mut self, core_state: &mut Core) {}

    pub fn apply_node_changes(&mut self, node_changes: &NodeChanges) {}
}

impl Transmitter for Device {
    type Item = Payload;
    fn collect_downstream(&mut self) -> Vec<Self::Item> {
        todo!()
    }

    fn generate_data(&mut self, core: &mut Core) -> Self::Item {
        todo!()
    }

    fn transmit(&mut self, data: Self::Item) {
        todo!()
    }
}

impl Node for Device {
    fn power_state(&self) -> PowerState {
        self.power_state
    }

    fn node_order(&self) -> i32 {
        self.node_info.order.as_i32()
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
}
