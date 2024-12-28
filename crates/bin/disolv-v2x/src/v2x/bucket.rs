use log::{debug, info};
use typed_builder::TypedBuilder;

use disolv_core::agent::{AgentClass, AgentId, AgentKind};
use disolv_core::bucket::Bucket;
use disolv_core::bucket::TimeMS;
use hashbrown::HashMap;
use disolv_core::metrics::Consumable;
use disolv_core::model::BucketModel;
use disolv_core::radio::Link;
use disolv_models::bucket::lake::DataLake;
use disolv_models::device::mobility::MapState;
use disolv_models::net::network::{Network, SliceType};
use disolv_models::net::radio::{CommStats, LinkProperties};
use disolv_output::result::Results;
use disolv_output::tables::net::NetStats;

use crate::models::message::{DataBlob, DataType, MessageType, TxMetrics};
use crate::models::message::PayloadInfo;
use crate::models::network::{Slice, V2XSlice};
use crate::v2x::device::DeviceInfo;
use crate::v2x::linker::Linker;
use crate::v2x::space::{Mapper, Space};

pub type V2XDataLake = DataLake<DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType>;
pub type V2XNetwork =
    Network<Slice, DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType, V2XSlice, TxMetrics>;

#[derive(TypedBuilder)]
pub struct BucketModels {
    pub network: V2XNetwork,
    pub results: Results,
    pub space: Space,
    pub mapper_holder: Vec<(AgentKind, Mapper)>,
    pub linker_holder: Vec<Linker>,
    pub stats_holder: HashMap<AgentId, CommStats>,
    pub device_infos: HashMap<AgentId, DeviceInfo>,
    pub data_lake: V2XDataLake,
}

#[derive(TypedBuilder)]
pub struct DeviceBucket {
    pub models: BucketModels,
    pub class_to_type: HashMap<AgentClass, AgentKind>,
    #[builder(default)]
    pub step: TimeMS,
}

impl DeviceBucket {
    pub fn link_options_for(
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

    pub fn positions_for(
        &mut self,
        agent_id: AgentId,
        device_type: &AgentKind,
    ) -> Option<MapState> {
        self.mapper_for(device_type).map_state_of(agent_id)
    }

    pub fn update_stats_of(&mut self, agent_id: AgentId, stats: CommStats) {
        self.models.stats_holder.insert(agent_id, stats);
    }

    pub fn stats_for(&self, agent_id: &AgentId) -> Option<&CommStats> {
        self.models.stats_holder.get(agent_id)
    }

    pub fn update_device_info_of(&mut self, agent_id: AgentId, d_info: DeviceInfo) {
        self.models.device_infos.insert(agent_id, d_info);
    }

    pub fn device_info_of(&self, agent_id: &AgentId) -> Option<&DeviceInfo> {
        self.models.device_infos.get(agent_id)
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

    fn mapper_for(&mut self, device_type: &AgentKind) -> &mut Mapper {
        self.models
            .mapper_holder
            .iter_mut()
            .find(|(n_type, _)| *n_type == *device_type)
            .map(|(_, mapper)| mapper)
            .expect("No mapper found for agent type")
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

    fn after_agents(&mut self) {
        for slice in self.models.network.slices.values() {
            if let Some(net_writer) = &mut self.models.results.net_stats {
                let net_stats = NetStats::builder()
                    .slice_id(slice.id)
                    .bandwidth(slice.resources.bandwidth_type.available().as_u64())
                    .build();
                net_writer.add_data(self.step, net_stats);
            }
        }
    }

    fn stream_input(&mut self) {
        self.models.mapper_holder.iter_mut().for_each(|(_, space)| {
            space.stream_data(self.step);
        });
        self.models.linker_holder.iter_mut().for_each(|linker| {
            linker.stream_data(self.step);
        });
    }

    fn stream_output(&mut self) {
        debug!("Writing output at {}", self.step);
        self.models.results.write_to_file();
    }

    fn terminate(mut self) {
        self.models.results.write_to_file();
        self.models.results.close_files();
    }
}
