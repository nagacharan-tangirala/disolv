use crate::linker::Linker;
use crate::space::{Mapper, Space};
use advaitars_core::agent::AgentId;
use advaitars_core::bucket::Bucket;
use advaitars_core::bucket::TimeMS;
use advaitars_core::hashbrown::HashMap;
use advaitars_core::model::BucketModel;
use advaitars_models::bucket::lake::DataLake;
use advaitars_models::device::mobility::MapState;
use advaitars_models::device::types::{DeviceClass, DeviceType};
use advaitars_models::net::network::Network;
use advaitars_models::net::radio::{DLink, OutgoingStats};
use advaitars_output::result::ResultWriter;
use log::info;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct BucketModels {
    pub result_writer: ResultWriter,
    pub network: Network,
    pub space: Space,
    pub mapper_holder: Vec<(DeviceType, Mapper)>,
    pub linker_holder: Vec<Linker>,
    #[builder(default)]
    pub data_lake: DataLake,
}

#[derive(Clone, TypedBuilder)]
pub struct DeviceBucket {
    pub models: BucketModels,
    pub class_to_type: HashMap<DeviceClass, DeviceType>,
    #[builder(default)]
    pub step: TimeMS,
}

impl DeviceBucket {
    pub(crate) fn link_options_for(
        &mut self,
        node_id: AgentId,
        source_type: &DeviceType,
        target_class: &DeviceClass,
    ) -> Option<Vec<DLink>> {
        match self.linker_for(source_type, target_class) {
            Some(linker) => linker.links_of(node_id),
            None => None,
        }
    }

    pub(crate) fn positions_for(
        &mut self,
        node_id: AgentId,
        node_type: &DeviceType,
    ) -> Option<MapState> {
        self.mapper_for(node_type).map_state_of(node_id)
    }

    fn linker_for(
        &mut self,
        source_type: &DeviceType,
        target_class: &DeviceClass,
    ) -> Option<&mut Linker> {
        let target_type = match self.class_to_type.get(target_class) {
            Some(t_type) => t_type,
            None => return None,
        };
        self.models
            .linker_holder
            .iter_mut()
            .find(|linker| linker.source_type == *source_type && linker.target_type == *target_type)
    }

    fn mapper_for(&mut self, node_type: &DeviceType) -> &mut Mapper {
        self.models
            .mapper_holder
            .iter_mut()
            .find(|(n_type, _)| *n_type == *node_type)
            .map(|(_, mapper)| mapper)
            .expect("No mapper found for node type")
    }
}

impl Bucket for DeviceBucket {
    fn initialize(&mut self, step: TimeMS) {
        self.step = step;
        self.models
            .mapper_holder
            .iter_mut()
            .for_each(|(_, mapper)| {
                mapper.init(self.step);
            });
        self.models.linker_holder.iter_mut().for_each(|linker| {
            linker.init(self.step);
        });
    }

    fn before_agents(&mut self, step: TimeMS) {
        self.step = step;
        info!("Before agents in bucket at step {}", step);
        self.models.network.reset_slices();

        self.models.data_lake.clean_payloads();
        self.models.data_lake.clean_responses();
        self.models
            .mapper_holder
            .iter_mut()
            .for_each(|(_, mapper)| {
                mapper.before_node_step(self.step);
            });
        self.models.linker_holder.iter_mut().for_each(|linker| {
            linker.before_node_step(self.step);
        });
    }

    fn after_agents(&mut self) {
        for slice in self.models.network.slices.iter() {
            self.models.result_writer.add_net_stats(self.step, slice);
        }
    }

    fn stream_input(&mut self, step: TimeMS) {
        self.models.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.stream_data(step);
        });
        self.models.linker_holder.iter_mut().for_each(|linker| {
            linker.stream_data(step);
        });
    }

    fn stream_output(&mut self, step: TimeMS) {
        self.models.result_writer.write_output(self.step);
    }

    fn terminate(&mut self, step: TimeMS) {
        self.models.result_writer.write_output(step);
    }
}
