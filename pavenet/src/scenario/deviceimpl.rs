use crate::scenario::episode::NodeChanges;
use crate::scenario::model::DeviceModel;
use pavenet_core::structs::{MapState, NodeInfo};
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_engine::engine::core::Core;
use pavenet_engine::node::node::Node;
use pavenet_engine::node::power::{PowerSchedule, PowerState};
use pavenet_models::node::payload::Payload;
use pavenet_models::node::traits::{Recipient, Transmitter};

#[derive(Clone, Copy, Debug, Default)]
pub enum DataState {
    On,
    #[default]
    Off,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub data_state: DataState,
    pub step: TimeStamp,
}

impl Device {
    pub fn new() -> DeviceBuilder {
        DeviceBuilder::default()
    }

    fn update_map_state(&mut self, core: &mut Core) {}

    fn collect_data(&mut self, core_state: &mut Core) {}

    pub fn apply_node_changes(&mut self, node_changes: &NodeChanges) {
        self.node_info.node_type = node_changes.new_node_type;
        self.node_info.node_class = node_changes.new_node_class;
        self.node_info.order = node_changes.new_order;
    }
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

impl Recipient for Device {
    type Item = Payload;
    fn receive(&mut self, data: &Vec<Self::Item>) {
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
        // self.receive(core);
        self.report_stats(core);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceBuilder {
    node_info: NodeInfo,
    models: DeviceModel,
}

impl DeviceBuilder {
    pub fn with_node_info(mut self, node_info: NodeInfo) -> Self {
        self.node_info = node_info;
        self
    }

    pub fn with_models(mut self, models: DeviceModel) -> Self {
        self.models = models;
        self
    }

    pub fn build(self) -> Device {
        Device {
            node_info: self.node_info,
            models: self.models,
            ..Default::default()
        }
    }
}
