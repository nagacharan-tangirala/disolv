use crate::device::Device;
use crate::linker::Linker;
use crate::space::{Mapper, Space};
use log::debug;
use pavenet_core::entity::{NodeClass, NodeOrder, NodeType};
use pavenet_core::mobility::MapState;
use pavenet_core::radio::{DLink, InDataStats};
use pavenet_engine::bucket::Bucket;
use pavenet_engine::bucket::ResultSaver;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::entity::Schedulable;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::node::NodeId;
use pavenet_engine::scheduler::GNodeScheduler;
use pavenet_models::lake::DataLake;
use pavenet_models::model::BucketModel;
use pavenet_output::result::ResultWriter;
use typed_builder::TypedBuilder;

pub type DNodeScheduler = GNodeScheduler<DeviceBucket, Device, NodeOrder>;

#[derive(Clone, TypedBuilder)]
pub struct DeviceBucket {
    pub space: Space,
    pub scheduler: DNodeScheduler,
    pub mapper_holder: Vec<(NodeType, Mapper)>,
    pub linker_holder: Vec<Linker>,
    pub class_to_type: HashMap<NodeClass, NodeType>,
    pub output_step: TimeMS,
    pub resultant: ResultWriter,
    #[builder(default)]
    pub step: TimeMS,
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
        source_type: &NodeType,
        target_class: &NodeClass,
    ) -> Option<Vec<DLink>> {
        match self.linker_for(source_type, target_class) {
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

    pub(crate) fn node_of(&mut self, node_id: NodeId) -> Option<&mut Device> {
        self.devices.get_mut(&node_id)
    }

    pub(crate) fn stats_for(&mut self, link_opts: &Vec<DLink>) -> Vec<Option<&InDataStats>> {
        let mut link_stats = Vec::with_capacity(link_opts.len());
        for link_opt in link_opts.iter() {
            link_stats.push(self.transfer_stats.get(&link_opt.target));
        }
        link_stats
    }

    pub fn stop_node(&mut self, node_id: NodeId) {
        self.scheduler.pop(node_id);
    }

    pub fn add_to_schedule(&mut self, node_id: NodeId) {
        self.scheduler.add(node_id);
    }

    fn linker_for(
        &mut self,
        source_type: &NodeType,
        target_class: &NodeClass,
    ) -> Option<&mut Linker> {
        let target_type = match self.class_to_type.get(target_class) {
            Some(t_type) => t_type,
            None => return None,
        };
        self.linker_holder
            .iter_mut()
            .find(|linker| linker.source_type == *source_type && linker.target_type == *target_type)
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
                .insert(*node_id, device.models.rx_radio.in_stats);
        });
    }
}

impl Bucket for DeviceBucket {
    type SchedulerImpl = DNodeScheduler;

    fn scheduler(&mut self) -> &mut DNodeScheduler {
        &mut self.scheduler
    }

    fn init(&mut self, step: TimeMS) {
        self.step = step;
        self.mapper_holder.iter_mut().for_each(|(_, mapper)| {
            mapper.init(self.step);
        });
        self.linker_holder.iter_mut().for_each(|linker| {
            linker.init(self.step);
        });
    }

    fn update(&mut self, step: TimeMS) {
        self.step = step;
        self.update_stats();
    }

    fn before_uplink(&mut self) {
        self.mapper_holder.iter_mut().for_each(|(_, mapper)| {
            mapper.before_node_step(self.step);
        });
        self.linker_holder.iter_mut().for_each(|linker| {
            linker.before_node_step(self.step);
        });
        // Clean up the messages
        self.data_lake.clean_up_payloads();
    }

    fn after_downlink(&mut self) {
        self.transfer_stats.clear();
        self.save_device_stats(self.step);
        self.save_data_stats(self.step);
        if self.step == self.output_step {
            self.resultant.write_output(self.step);
            self.output_step += self.output_step;
        }
        // Clean up the responses
        self.data_lake.clean_up_responses();
    }

    fn streaming_step(&mut self, step: TimeMS) {
        self.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.stream_data(step);
        });
        self.linker_holder.iter_mut().for_each(|linker| {
            linker.stream_data(step);
        });
    }

    fn end(&mut self, step: TimeMS) {
        self.resultant.write_output(step);
    }
}

impl ResultSaver for DeviceBucket {
    fn save_device_stats(&mut self, step: TimeMS) {
        for (node_id, device) in self.devices.iter() {
            if device.is_stopped() {
                continue;
            }
            self.resultant
                .add_node_pos(step, *node_id, &device.map_state);
        }
    }

    fn save_data_stats(&mut self, step: TimeMS) {
        for (node_id, device) in self.devices.iter() {
            if device.is_stopped() {
                continue;
            }
            self.resultant
                .add_rx_counts(step, *node_id, &device.models.rx_radio.in_stats);
        }
    }
}
