use crate::node::device::Device;
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use log::error;
use pavenet_core::types::{NodeId, Order, TimeStamp};
use pavenet_engine::node::pool::NodePool;
use pavenet_models::node_pool::episode::Episode;
use pavenet_models::node_pool::space::Space;
use pavenet_models::node_pool::vanet::Vanet;

pub struct Devices {
    devices: Vec<Device>,
    by_class: HashMap<u32, NodeId>,
    by_order: HashMap<Order, NodeId>,
    pub(crate) to_add: Vec<(NodeId, TimeStamp)>,
    pub(crate) to_pop: Vec<NodeId>,
    pub(crate) space: Space,
    pub vanet: Vanet,
    pub(crate) episode: Episode,
}

impl Devices {
    pub fn new(space: Space, vanet: Vanet, episode: Episode) -> Self {
        Self {
            space,
            vanet,
            episode,
            ..Default::default()
        }
    }
}

impl NodePool for Devices {
    fn init(&mut self, schedule: &mut Schedule) {
        for device in self.devices.iter() {
            self.by_class
                .insert(device.node_info.node_class, device.node_info.id);
            self.by_order
                .insert(device.node_info.order, device.node_info.id);
        }
        self.space.init(TimeStamp::from(schedule.step));
    }

    fn before_step(&mut self, step: TimeStamp) {
        self.space.refresh_cache(step);
    }

    fn update(&mut self, step: TimeStamp) {
        todo!()
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        todo!()
    }

    fn streaming_step(&mut self, step: TimeStamp) {
        self.space.stream_data(step);
    }
}
