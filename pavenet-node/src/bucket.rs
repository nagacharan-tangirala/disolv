use crate::d_model::BucketModel;
use crate::device::Device;
use crate::models::lake::DataLake;
use crate::models::linker::Linker;
use crate::models::space::{Mapper, Space};
use pavenet_core::bucket::TimeS;
use pavenet_core::entity::class::NodeClass;
use pavenet_core::entity::id::NodeId;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::link::{DLink, DLinkOptions};
use pavenet_core::mobility::MapState;
use pavenet_core::radio::stats::InDataStats;
use pavenet_core::rules::Rules;
use pavenet_engine::bucket::Bucket;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::scheduler::GNodeScheduler;
use typed_builder::TypedBuilder;

pub type DNodeScheduler = GNodeScheduler<DeviceBucket, Device, NodeId, NodeType, NodeClass, TimeS>;

#[derive(Clone, TypedBuilder)]
pub struct DeviceBucket {
    pub space: Space,
    pub rules: Rules,
    pub scheduler: DNodeScheduler,
    pub mapper_holder: Vec<(NodeType, Mapper)>,
    pub linker_holder: Vec<(NodeType, Linker)>,
    pub class_to_type: HashMap<NodeClass, NodeType>,
    #[builder(default)]
    pub step: TimeS,
    #[builder(default)]
    pub data_lake: DataLake,
    #[builder(default)]
    pub(crate) devices: HashMap<NodeId, Device>,
    #[builder(default)]
    transfer_stats: HashMap<NodeId, InDataStats>,
}

impl DeviceBucket {
    pub(crate) fn link_options_for(
        &mut self,
        node_id: NodeId,
        target_type: &NodeType,
    ) -> Option<DLinkOptions> {
        match self.linker_for(target_type) {
            Some(linker) => linker.links_of(node_id),
            None => None,
        }
    }

    pub(crate) fn positions_for(
        &mut self,
        node_id: NodeId,
        node_type: &NodeType,
    ) -> Option<MapState> {
        self.mapper_for(node_type).map_state_of(node_id)
    }

    pub(crate) fn stats_for(&mut self, link_opts: &Vec<DLink>) -> Vec<Option<&InDataStats>> {
        let mut link_stats = Vec::with_capacity(link_opts.len());
        for link_opt in link_opts.iter() {
            link_stats.push(self.transfer_stats.get(&link_opt.target));
        }
        link_stats
    }

    pub(crate) fn kind_for(&self, target_class: &NodeClass) -> NodeType {
        match self.class_to_type.get(target_class) {
            Some(node_type) => node_type.to_owned(),
            None => panic!("No node type for class: {:?}", target_class),
        }
    }

    pub fn stop_node(&mut self, node_id: NodeId) {
        self.scheduler.pop(node_id);
    }

    pub fn add_to_schedule(&mut self, node_id: NodeId) {
        self.scheduler.add(node_id);
    }

    fn linker_for(&mut self, target_type: &NodeType) -> Option<&mut Linker> {
        self.linker_holder
            .iter_mut()
            .find(|(node_type, _)| *node_type == *target_type)
            .map(|(_, linker)| linker)
    }

    fn mapper_for(&mut self, node_type: &NodeType) -> &mut Mapper {
        self.mapper_holder
            .iter_mut()
            .find(|(n_type, _)| *n_type == *node_type)
            .map(|(_, mapper)| mapper)
            .expect("No mapper for node type")
    }

    fn update_stats(&mut self) {
        self.devices.iter().for_each(|(node_id, device)| {
            self.transfer_stats
                .insert(*node_id, device.models.radio.in_stats.clone());
        });
    }
}

impl Bucket<TimeS> for DeviceBucket {
    type SchedulerImpl = DNodeScheduler;

    fn scheduler(&mut self) -> &mut DNodeScheduler {
        &mut self.scheduler
    }

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
        self.update_stats();
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
