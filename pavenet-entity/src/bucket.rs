use crate::d_model::BucketModel;
use crate::device::Device;
use crate::models::channel::InDataStats;
use crate::models::lake::DataLake;
use crate::models::linker::Linker;
use crate::models::space::{Mapper, Space};
use hashbrown::HashMap;
use pavenet_core::mobility::MapState;
use pavenet_core::node_info::id::NodeId;
use pavenet_core::node_info::kind::NodeType;
use pavenet_core::node_info::order::Order;
use pavenet_core::times::ts::TimeS;
use pavenet_engine::bucket::Bucket;
use pavenet_engine::scheduler::NodeScheduler;
use typed_builder::TypedBuilder;

pub type EScheduler = NodeScheduler<DeviceBucket, Device, NodeId, NodeType, Order, TimeS>;

#[derive(Clone, TypedBuilder)]
pub struct DeviceBucket {
    pub(crate) devices: HashMap<NodeId, Device>,
    transfer_stats: HashMap<NodeId, InDataStats>,
    pub mapper_holder: Vec<(NodeType, Mapper)>,
    pub linker_holder: Vec<(NodeType, Linker)>,
    pub entity_scheduler: EScheduler,
    pub data_lake: DataLake,
    pub space: Space,
    pub step: TimeS,
}

impl DeviceBucket {
    pub(crate) fn links_for(&mut self, node_id: NodeId, target_type: &NodeType) -> Option<Link> {
        self.linker_for(target_type).links_of(node_id)
    }

    pub(crate) fn positions_for(
        &mut self,
        node_id: NodeId,
        node_type: &NodeType,
    ) -> Option<MapState> {
        self.mapper_for(node_type).map_state_of(node_id)
    }

    pub(crate) fn stats_for(&mut self, links: &Link) -> Vec<Option<&InDataStats>> {
        let mut link_stats = Vec::with_capacity(links.target.len());
        for target in links.target.iter() {
            link_stats.push(self.transfer_stats.get(target));
        }
        link_stats
    }

    pub fn stop_node(&mut self, node_id: NodeId) {
        self.entity_scheduler.pop(node_id);
    }

    pub fn add_to_schedule(&mut self, node_id: NodeId) {
        self.entity_scheduler.add(node_id);
    }

    fn linker_for(&mut self, target_type: &NodeType) -> &mut Linker {
        self.linker_holder
            .iter_mut()
            .find(|(node_type, _)| *node_type == *target_type)
            .map(|(_, links)| links)
            .unwrap_or_else(|| panic!("No Linker for node type: {:?}", target_type))
    }

    fn mapper_for(&mut self, node_type: &NodeType) -> &mut Mapper {
        self.mapper_holder
            .iter_mut()
            .find(|(node_type, _)| *node_type == *node_type)
            .map(|(_, space)| space)
            .unwrap_or_else(|| panic!("No mapper for node type: {:?}", node_type))
    }

    fn update_stats(&mut self) {
        self.devices.iter().for_each(|(node_id, device)| {
            self.transfer_stats
                .insert(*node_id, device.models.channel.in_stats.clone());
        });
    }
}

impl Bucket<TimeS> for DeviceBucket {
    fn init(&mut self, step: TimeS) {
        self.step = step;
        self.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.init(self.step);
        });
        self.linker_holder.iter_mut().for_each(|(_, linker)| {
            linker.init(self.step);
        });
    }

    fn update(&mut self, step: TimeS) {
        self.step = step;
    }

    fn before_uplink(&mut self) {
        self.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.refresh_cache(self.step);
        });
        self.linker_holder.iter_mut().for_each(|(_, linker)| {
            linker.refresh_cache(self.step);
        });
    }

    fn after_downlink(&mut self) {
        self.transfer_stats.clear();
    }

    fn streaming_step(&mut self, step: TimeS) {
        self.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.stream_data(step);
        });
        self.linker_holder.iter_mut().for_each(|(_, linker)| {
            linker.stream_data(step);
        });
    }
}
