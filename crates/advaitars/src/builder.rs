use crate::base::{AgentClassSettings, AgentSettings, BaseConfig, BaseConfigReader};
use crate::logger;
use advaitars_core::agent::{Agent, AgentId, AgentImpl, AgentOrder};
use advaitars_core::bucket::TimeMS;
use advaitars_core::core::Core;
use advaitars_core::hashbrown::HashMap;
use advaitars_core::metrics::{Consumable, Measurable};
use advaitars_core::model::Model;
use advaitars_core::scheduler::DefaultScheduler;
use advaitars_device::bucket::{BucketModels, DeviceBucket};
use advaitars_device::device::{Device, DeviceModel};
use advaitars_device::linker::{Linker, LinkerSettings};
use advaitars_device::space::{Mapper, Space};
use advaitars_input::links::LinkReader;
use advaitars_input::power::{read_power_schedule, PowerTimes};
use advaitars_models::bucket::flow::FlowRegister;
use advaitars_models::bucket::lake::DataLake;
use advaitars_models::device::actor::Actor;
use advaitars_models::device::compose::Composer;
use advaitars_models::device::power::PowerManager;
use advaitars_models::device::reply::Replier;
use advaitars_models::device::select::Selector;
use advaitars_models::device::types::{DeviceClass, DeviceInfo, DeviceType};
use advaitars_models::net::bandwidth::BandwidthType;
use advaitars_models::net::latency::LatencyType;
use advaitars_models::net::metrics::RadioMetricTypes;
use advaitars_models::net::network::Network;
use advaitars_models::net::slice::{RadioMetrics, RadioResources, Slice, SliceSettings};
use advaitars_output::result::ResultWriter;
use log::{debug, info};
use std::path::{Path, PathBuf};

pub type DCore = Core<Device, DeviceBucket>;
pub type DScheduler = DefaultScheduler<Device, DeviceBucket>;
pub type DAgentImpl = AgentImpl<Device, DeviceBucket>;

pub struct SimulationBuilder {
    base_config: BaseConfig,
    config_path: PathBuf,
}

impl SimulationBuilder {
    pub(crate) fn new(base_config_file: &str) -> Self {
        if !Path::new(base_config_file).exists() {
            panic!("Configuration file is not found.");
        }
        let config_path = Path::new(base_config_file)
            .parent()
            .unwrap_or_else(|| {
                panic!("Invalid directory for the configuration file");
            })
            .to_path_buf();

        let config_reader = BaseConfigReader::new(&base_config_file);
        match config_reader.parse() {
            Ok(base_config) => Self {
                base_config,
                config_path,
            },
            Err(e) => {
                panic!("Error while parsing the base configuration file: {}", e);
            }
        }
    }

    pub(crate) fn build(&mut self) -> DScheduler {
        logger::initiate_logger(&self.config_path, &self.base_config.log_settings);

        info!("Building devices and device pools...");
        let device_bucket = self.build_device_bucket();
        let agent_map = self.build_agents();
        self.build_scheduler(agent_map, device_bucket)
    }

    fn read_power_schedules(&self, device_type: DeviceType) -> HashMap<AgentId, PowerTimes> {
        let device_settings: &AgentSettings = self
            .base_config
            .agents
            .iter()
            .find(|x| x.agent_type == device_type)
            .unwrap_or_else(|| panic!("Invalid device type: {}", device_type));

        let power_file = Path::new(&self.config_path).join(&device_settings.power_file);
        if !power_file.exists() {
            panic!("Power schedule file {} is not found.", power_file.display());
        }
        read_power_schedule(&power_file)
    }

    fn read_device_ids(power_schedules: &HashMap<AgentId, PowerTimes>) -> Vec<AgentId> {
        let mut device_ids: Vec<AgentId> = Vec::new();
        power_schedules.iter().for_each(|(device_id, _)| {
            device_ids.push(*device_id);
        });
        device_ids
    }

    fn build_agents(&mut self) -> HashMap<AgentId, DAgentImpl> {
        info!("Building devices...");
        let mut device_map = HashMap::new();

        for device_setting in self.base_config.agents.clone().iter() {
            let mut power_schedules = self.read_power_schedules(device_setting.agent_type);
            let device_ids = Self::read_device_ids(&power_schedules);
            let device_count = device_ids.len();
            info!(
                "Building devices for device type: {}",
                device_setting.agent_type
            );

            for class_settings in device_setting.class.iter() {
                let class_count = (class_settings.agent_share * device_count as f32) as usize;
                let mut device_count = 0;

                for device_id in device_ids.iter() {
                    let device_schedule = power_schedules
                        .remove(device_id)
                        .unwrap_or_else(|| panic!("Invalid device id"));
                    let device = self.build_device(
                        *device_id,
                        &device_setting.agent_type,
                        class_settings,
                        device_schedule,
                    );
                    let agent_impl = AgentImpl::builder()
                        .agent_id(*device_id)
                        .agent(device)
                        .build();
                    device_map.insert(*device_id, agent_impl);
                    device_count += 1;
                    if device_count == class_count {
                        break;
                    }
                }
            }
        }
        return device_map;
    }

    fn build_device(
        &mut self,
        device_id: AgentId,
        device_type: &DeviceType,
        class_settings: &AgentClassSettings,
        power_times: PowerTimes,
    ) -> Device {
        let device_info = Self::build_device_info(device_id, device_type, class_settings);

        let power_manager = PowerManager::builder()
            .on_times(power_times.0.into())
            .off_times(power_times.1.into())
            .array_idx(0)
            .build();

        let mut selector_vec = Vec::new();
        class_settings.selector.iter().for_each(|settings| {
            let selector = Selector::with_settings(settings);
            selector_vec.push((settings.target_class, selector));
        });

        let device_model = DeviceModel::builder()
            .power(power_manager)
            .flow(FlowRegister::default())
            .sl_flow(FlowRegister::default())
            .composer(Composer::with_settings(&class_settings.composer))
            .selector(selector_vec)
            .actor(Actor::new(&class_settings.actions.clone()))
            .replier(Replier::with_settings(&class_settings.replier))
            .build();

        let device = Device::builder()
            .device_info(device_info)
            .models(device_model)
            .build();

        return device;
    }

    fn build_device_info(
        device_id: AgentId,
        device_type: &DeviceType,
        class_settings: &AgentClassSettings,
    ) -> DeviceInfo {
        DeviceInfo::builder()
            .id(device_id)
            .device_type(device_type.to_owned())
            .device_class(class_settings.agent_class)
            .agent_order(class_settings.agent_order)
            .build()
    }

    fn build_scheduler(
        &mut self,
        agent_map: HashMap<AgentId, DAgentImpl>,
        device_bucket: DeviceBucket,
    ) -> DScheduler {
        info!("Building scheduler...");
        DefaultScheduler::builder()
            .duration(self.duration())
            .step_size(self.step_size())
            .agents(agent_map)
            .core(DCore::new(device_bucket))
            .streaming_interval(self.streaming_interval())
            .output_interval(self.output_interval())
            .build()
    }

    fn build_device_bucket(&mut self) -> DeviceBucket {
        info!("Building device bucket...");
        DeviceBucket::builder()
            .models(self.build_bucket_models())
            .class_to_type(self.read_class_to_type_map())
            .build()
    }

    fn build_bucket_models(&mut self) -> BucketModels {
        BucketModels::builder()
            .result_writer(ResultWriter::new(&self.base_config.output_settings))
            .network(self.build_network())
            .space(self.build_space())
            .mapper_holder(self.build_mapper_vec())
            .linker_holder(self.build_linker_vec())
            .data_lake(DataLake::default())
            .build()
    }

    fn build_mapper_vec(&self) -> Vec<(DeviceType, Mapper)> {
        let mut mapper_vec: Vec<(DeviceType, Mapper)> = Vec::new();
        for device_setting in self.base_config.agents.iter() {
            let device_type = device_setting.agent_type;
            let mapper = Mapper::builder(&self.config_path)
                .streaming_step(self.streaming_interval())
                .field_settings(self.base_config.field_settings.clone())
                .space_settings(device_setting.mobility.clone())
                .build();
            mapper_vec.push((device_type, mapper));
        }
        mapper_vec
    }

    fn build_linker_vec(&self) -> Vec<Linker> {
        let mut linker_vec: Vec<Linker> = Vec::new();
        for device_setting in self.base_config.agents.iter() {
            let device_type = device_setting.agent_type;
            match device_setting.linker {
                Some(ref linker_settings) => {
                    for link_setting in linker_settings.iter() {
                        let linker = self.build_linker(&device_type, link_setting);
                        linker_vec.push(linker);
                    }
                }
                None => {}
            };
        }
        linker_vec
    }

    fn build_linker(&self, source_type: &DeviceType, link_config: &LinkerSettings) -> Linker {
        let links_file = self.config_path.join(&link_config.links_file);
        if !links_file.exists() {
            panic!("Link file {} is not found.", links_file.display());
        }
        let link_reader = LinkReader::builder()
            .is_streaming(link_config.is_streaming)
            .file_path(links_file)
            .streaming_step(self.streaming_interval())
            .build();
        Linker::builder()
            .reader(link_reader)
            .source_type(source_type.to_owned())
            .target_type(link_config.target_type.to_owned())
            .is_static(!link_config.is_streaming)
            .build()
    }

    fn build_space(&self) -> Space {
        Space::builder()
            .height(self.base_config.field_settings.height)
            .cell_size(self.base_config.field_settings.cell_size)
            .width(self.base_config.field_settings.width)
            .build()
    }

    fn read_class_to_type_map(&mut self) -> HashMap<DeviceClass, DeviceType> {
        let mut class_to_type: HashMap<DeviceClass, DeviceType> = HashMap::new();
        for device_setting in self.base_config.agents.iter() {
            let device_classes: Vec<DeviceClass> =
                device_setting.class.iter().map(|x| x.agent_class).collect();
            for device_class in device_classes.iter() {
                class_to_type.insert(device_class.to_owned(), device_setting.agent_type);
            }
        }
        class_to_type
    }

    fn build_network(&self) -> Network {
        let mut slices: Vec<Slice> = Vec::new();
        for slice_setting in self.base_config.network_settings.slice.iter() {
            let slice = Slice::builder()
                .id(slice_setting.id)
                .name(slice_setting.name.clone())
                .step_size(self.step_size())
                .resources(self.build_network_resources(slice_setting))
                .metrics(self.build_network_metrics(slice_setting))
                .build();
            slices.push(slice);
        }
        Network::builder().slices(slices).build()
    }

    fn build_network_resources(&self, slice_settings: &SliceSettings) -> RadioResources {
        RadioResources::builder()
            .bandwidth_type(BandwidthType::with_settings(
                slice_settings.bandwidth.clone(),
            ))
            .build()
    }

    fn build_network_metrics(&self, slice_settings: &SliceSettings) -> RadioMetrics {
        RadioMetrics::builder()
            .latency_type(LatencyType::with_settings(slice_settings.latency.clone()))
            .build()
    }

    fn streaming_interval(&self) -> TimeMS {
        return self.base_config.simulation_settings.streaming_interval;
    }

    pub(crate) fn duration(&self) -> TimeMS {
        return self.base_config.simulation_settings.duration;
    }

    pub(crate) fn step_size(&self) -> TimeMS {
        return self.base_config.simulation_settings.step_size;
    }

    fn sim_seed(&self) -> u128 {
        return u128::from(self.base_config.simulation_settings.seed);
    }

    fn output_interval(&self) -> TimeMS {
        return self.base_config.output_settings.output_interval;
    }
}
