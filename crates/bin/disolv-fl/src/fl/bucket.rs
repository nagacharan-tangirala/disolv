use burn::data::dataset::transform::Mapper;
use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;
use log::info;
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId, AgentKind};
use disolv_core::bucket::{Bucket, TimeMS};
use disolv_core::hashbrown::HashMap;
use disolv_core::model::BucketModel;
use disolv_core::radio::Link;
use disolv_models::bucket::lake::DataLake;
use disolv_models::device::mobility::MapState;
use disolv_models::net::network::Network;
use disolv_models::net::radio::{CommStats, LinkProperties};
use disolv_output::result::ResultWriter;

use crate::fl::client::AgentInfo;
use crate::models::device::lake::ModelLake;
use crate::models::device::linker::Linker;
use crate::models::device::mapper::{GeoMap, GeoMapper};
use crate::models::device::message::{FlPayloadInfo, Message, MessageType, MessageUnit, TxMetrics};
use crate::models::device::network::{FlSlice, Slice};
use crate::models::device::output::OutputWriter;

pub type FlDataLake = DataLake<MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message>;
pub type FlNetwork =
    Network<Slice, MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message, FlSlice, TxMetrics>;

#[derive(TypedBuilder)]
pub(crate) struct FlModels<B: Backend> {
    pub(crate) output: OutputWriter,
    pub(crate) network: FlNetwork,
    pub(crate) space: GeoMap,
    pub(crate) mapper_holder: Vec<(AgentKind, GeoMapper)>,
    pub(crate) linker_holder: Vec<Linker>,
    pub(crate) stats_holder: HashMap<AgentId, CommStats>,
    pub(crate) agent_data: HashMap<AgentId, AgentInfo>,
    pub(crate) data_lake: FlDataLake,
    pub(crate) model_lake: ModelLake<B>,
}

#[derive(TypedBuilder)]
pub(crate) struct FlBucket<B: Backend> {
    pub(crate) models: FlModels<B>,
    pub class_to_type: HashMap<AgentClass, AgentKind>,
    #[builder(default)]
    pub(crate) step: TimeMS,
}

impl<B: Backend> FlBucket<B> {
    pub(crate) fn link_options_for(
        &mut self,
        agent_id: AgentId,
        source_type: &AgentKind,
        target_class: &AgentClass,
    ) -> Option<Vec<Link<LinkProperties>>> {
        match self.linker_for(source_type, target_class) {
            Some(linker) => linker.links_of(agent_id),
            None => None,
        }
    }

    pub(crate) fn positions_for(
        &mut self,
        agent_id: AgentId,
        device_type: &AgentKind,
    ) -> Option<MapState> {
        self.geo_mapper_for(device_type).map_state_of(agent_id)
    }

    pub fn update_stats_of(&mut self, agent_id: AgentId, stats: CommStats) {
        self.models.stats_holder.insert(agent_id, stats);
    }

    pub fn stats_for(&self, agent_id: &AgentId) -> Option<&CommStats> {
        self.models.stats_holder.get(agent_id)
    }

    pub fn update_agent_data_of(&mut self, agent_id: AgentId, a_info: AgentInfo) {
        self.models.agent_data.insert(agent_id, a_info);
    }

    pub fn agent_data_of(&self, agent_id: &AgentId) -> Option<&AgentInfo> {
        self.models.agent_data.get(agent_id)
    }

    fn linker_for(
        &mut self,
        source_type: &AgentKind,
        target_class: &AgentClass,
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

    fn geo_mapper_for(&mut self, device_type: &AgentKind) -> &mut GeoMapper {
        self.models
            .mapper_holder
            .iter_mut()
            .find(|(n_type, _)| *n_type == *device_type)
            .map(|(_, mapper)| mapper)
            .expect("No geo mapper found for agent type")
    }
}

impl<B: Backend> Bucket for FlBucket<B> {
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
        self.models
            .mapper_holder
            .iter_mut()
            .for_each(|(_, mapper)| {
                mapper.before_agent_step(self.step);
            });
        self.models.linker_holder.iter_mut().for_each(|linker| {
            linker.before_agent_step(self.step);
        });
    }

    fn after_stage_one(&mut self) {
        todo!()
    }

    fn after_stage_two(&mut self) {
        todo!()
    }

    fn after_stage_three(&mut self) {
        todo!()
    }

    fn after_stage_four(&mut self) {
        todo!()
    }

    fn after_agents(&mut self) {
        todo!()
    }

    fn stream_input(&mut self) {
        todo!()
    }

    fn stream_output(&mut self) {
        todo!()
    }

    fn terminate(self) {
        todo!()
    }
}
