use crate::config::base::{BaseConfig, BaseConfigReader, NodeClassSettings, NodeSettings};
use crate::config::episodes::EpisodeReader;
use crate::config::logger;
use crate::scenario::deviceimpl::Device;
use crate::scenario::devices::DevicePool;
use crate::scenario::episode::{Episode, EpisodeInfo};
use crate::scenario::model::DeviceModel;
use hashbrown::HashMap;
use log::info;
use pavenet_core::enums::NodeType;
use pavenet_core::named::class::Class;
use pavenet_core::structs::NodeInfo;
use pavenet_core::types::{NodeId, Order, PowerTimes, TimeStamp};
use pavenet_engine::engine::engine::Engine;
use pavenet_engine::engine::poolimpl::PoolImpl;
use pavenet_engine::node::node::Node;
use pavenet_engine::node::pool::NodePool;
use pavenet_engine::node::power::{PowerSchedule, SCHEDULE_SIZE};
use pavenet_input::input::links::{LinkReaderType, ReadLinks, StreamLinks};
use pavenet_input::input::power;
use pavenet_models::pool::linker::{Linker, LinkerSettings, NodeLinks};
use pavenet_models::pool::space::{Space, SpaceSettings};
use std::path::{Path, PathBuf};

pub struct PavenetBuilder {
    base_config: BaseConfig,
    config_path: PathBuf,
    streaming_step: TimeStamp,
    episodes: HashMap<NodeType, Vec<EpisodeInfo>>,
    power_schedules: HashMap<NodeType, HashMap<NodeId, PowerSchedule>>,
    node_ids_by_type: HashMap<NodeType, Vec<NodeId>>,
}

impl PavenetBuilder {
    pub fn new(base_config_file: &str) -> Self {
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
                streaming_step: TimeStamp::default(),
                episodes: HashMap::new(),
                power_schedules: HashMap::new(),
                node_ids_by_type: HashMap::new(),
            },
            Err(e) => {
                panic!("Error while parsing the base configuration file: {}", e);
            }
        }
    }

    pub fn build(&mut self) -> Engine {
        logger::initiate_logger(&self.config_path, &self.base_config.log_settings);

        let start_time = TimeStamp::default();
        let end_time = TimeStamp::from(self.base_config.simulation_settings.sim_duration);
        self.streaming_step =
            TimeStamp::from(self.base_config.simulation_settings.sim_streaming_step);

        info!("Reading power schedules...");
        self.read_power_schedules();
        self.read_node_ids();
        self.read_episodes();

        info!("Building devices and device pools...");
        let device_map = self.build_devices();
        let device_pools = self.build_device_pools(&device_map);

        let node_map = self.build_dyn_nodes(&device_map);
        let pool_impl = self.build_pool_impl(node_map);

        info!("Initializing Engine...");
        Engine::builder()
            .step(start_time)
            .end_step(end_time)
            .streaming_step(self.streaming_step)
            .pool_impl(pool_impl)
            .node_pools(device_pools)
            .build()
    }

    fn read_power_schedules(&mut self) {
        for node_setting in self.base_config.nodes.iter() {
            let node_type = node_setting.node_type;
            let power_schedule = self.read_power_schedule(node_setting);
            self.power_schedules
                .entry(node_type)
                .or_insert_with(HashMap::new)
                .extend(power_schedule);
        }
    }

    fn read_power_schedule(&self, node_setting: &NodeSettings) -> HashMap<NodeId, PowerSchedule> {
        let power_file = Path::new(&self.config_path).join(&node_setting.power_file);
        if !power_file.exists() {
            panic!("Power schedule file is not found.");
        }
        let power_times: HashMap<NodeId, PowerTimes> = match power::read_power_schedule(&power_file)
        {
            Ok(power_schedule) => power_schedule,
            Err(e) => {
                panic!("Error while parsing the power schedule file: {}", e);
            }
        };
        let power_schedule = self.create_power_schedules(power_times);
        return power_schedule;
    }

    fn create_power_schedules(
        &self,
        power_times: HashMap<NodeId, PowerTimes>,
    ) -> HashMap<NodeId, PowerSchedule> {
        let mut schedule_map = HashMap::new();
        for (node_id, power_times) in power_times.into_iter() {
            let mut on_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
            let mut off_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
            let on_vec = power_times.0;
            let off_vec = power_times.1;
            for (i, on_time) in on_vec.into_iter().enumerate() {
                on_times[i] = Some(on_time);
            }
            for (i, off_time) in off_vec.into_iter().enumerate() {
                off_times[i] = Some(off_time);
            }
            let power_schedule = PowerSchedule::new(on_times, off_times);
            schedule_map.insert(node_id, power_schedule);
        }
        return schedule_map;
    }

    fn read_node_ids(&mut self) {
        for (node_type, power_schedule) in self.power_schedules.iter() {
            self.node_ids_by_type
                .entry(node_type.clone())
                .or_insert_with(Vec::new)
                .extend(power_schedule.keys());
        }
    }

    fn read_episodes(&mut self) {
        if let Some(episode_settings) = &self.base_config.episode_settings {
            let episode_file = Path::new(&self.config_path).join(&episode_settings.episode_file);
            if !episode_file.exists() {
                return;
            }
            let episode_reader = EpisodeReader::new(episode_file);
            let episodes = match episode_reader.parse() {
                Ok(episode_list) => episode_list,
                Err(e) => {
                    panic!("Error while parsing the episode file: {}", e);
                }
            };
            for episode_info in episodes.episodes.into_iter() {
                let node_type = episode_info.node_config.node_type;
                self.episodes
                    .entry(node_type)
                    .or_insert_with(Vec::new)
                    .extend(vec![episode_info]);
            }
        }
    }

    fn build_devices(&mut self) -> HashMap<NodeId, Device> {
        info!("Building devices...");
        let mut device_map = HashMap::new();
        for node_setting in self.base_config.nodes.iter() {
            let node_type = node_setting.node_type;
            let node_ids = self
                .node_ids_by_type
                .get(&node_type)
                .expect("Invalid node type.");
            let node_count = node_ids.len();

            info!("Building devices for node type: {}", node_type);
            for class_settings in node_setting.class.iter() {
                let class_count = (class_settings.node_share * node_count as f32) as usize;
                let mut device_count = 0;

                for node_id in node_ids.iter() {
                    let device = self.build_device(*node_id, node_type, class_settings);
                    device_map.insert(*node_id, device);
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
        &self,
        node_id: NodeId,
        node_type: NodeType,
        class_settings: &NodeClassSettings,
    ) -> Device {
        let node_info = NodeInfo::builder()
            .id(node_id)
            .node_type(node_type)
            .node_class(Class::from(class_settings.node_class))
            .order(Order::from(class_settings.node_order))
            .build();

        let models: DeviceModel = DeviceModel::builder()
            .with_composer(&class_settings.composer)
            .with_simplifier(&class_settings.simplifier)
            .build();

        let device = Device::new()
            .with_node_info(node_info)
            .with_models(models)
            .build();

        return device;
    }

    fn build_device_pools(&mut self, devices: &HashMap<NodeId, Device>) -> Vec<Box<dyn NodePool>> {
        let mut device_pools = Vec::new();
        for node_setting in self.base_config.nodes.iter() {
            let space = self.build_space(&node_setting.space);
            let mut node_links = NodeLinks::default();
            let mut targets = Vec::new();
            match &node_setting.linker {
                Some(linker_settings) => {
                    node_links = self.build_node_links(linker_settings);
                    targets = self.collect_targets(linker_settings);
                }
                None => {}
            };

            let episodes = match self.episodes.get(&node_setting.node_type) {
                Some(episodes) => episodes.clone(),
                None => Vec::new(),
            };

            let episode = Episode::new(&episodes);

            let this_device_map = self.get_devices_of_type(&node_setting.node_type, devices);
            let device_pool = DevicePool::builder()
                .devices(this_device_map)
                .episode(episode)
                .space(space)
                .node_links(node_links)
                .targets(targets)
                .build();
            let dyn_pool: Box<dyn NodePool> = Box::new(device_pool);
            device_pools.push(dyn_pool);
        }
        return device_pools;
    }

    fn build_space(&self, mobility: &SpaceSettings) -> Space {
        let space = Space::builder(&self.config_path)
            .field_settings(self.base_config.field_settings.clone())
            .space_settings(mobility.clone())
            .streaming_step(self.streaming_step)
            .build();
        return space;
    }

    fn build_node_links(&self, linker_settings: &Vec<LinkerSettings>) -> NodeLinks {
        let mut node_type_linkers: Vec<(NodeType, Linker)> = Vec::new();
        for link_config in linker_settings.iter() {
            let linker = self.build_linker(&link_config);
            node_type_linkers.push((link_config.target_type, linker));
        }
        return NodeLinks::new(node_type_linkers);
    }

    fn build_linker(&self, link_config: &LinkerSettings) -> Linker {
        let links_file = self.config_path.join(&link_config.links_file);
        let link_reader = match link_config.is_streaming {
            true => LinkReaderType::Stream(
                StreamLinks::builder()
                    .links_file(links_file)
                    .streaming_interval(self.streaming_step)
                    .build(),
            ),
            false => LinkReaderType::File(ReadLinks::builder().links_file(links_file).build()),
        };
        return Linker::new(link_config.clone(), link_reader);
    }

    fn collect_targets(&self, link_configs: &Vec<LinkerSettings>) -> Vec<NodeType> {
        let mut targets = Vec::new();
        for link_config in link_configs {
            targets.push(link_config.target_type);
        }
        return targets;
    }

    fn get_devices_of_type(
        &self,
        node_type: &NodeType,
        devices: &HashMap<NodeId, Device>,
    ) -> HashMap<NodeId, Device> {
        let node_ids = self
            .node_ids_by_type
            .get(node_type)
            .unwrap_or_else(|| panic!("Invalid node type"));

        let mut this_device_map = HashMap::with_capacity(node_ids.len());
        for node_id in node_ids.iter() {
            let device = devices
                .get(node_id)
                .unwrap_or_else(|| panic!("Device is not found for node_id: {}", node_id.as_u32()));
            this_device_map.insert(*node_id, device.to_owned());
        }
        return this_device_map;
    }

    fn build_dyn_nodes(&self, devices: &HashMap<NodeId, Device>) -> HashMap<NodeId, Box<dyn Node>> {
        let mut dyn_nodes: HashMap<NodeId, Box<dyn Node>> = HashMap::with_capacity(devices.len());
        for (node_id, device) in devices.iter() {
            let dyn_node: Box<dyn Node> = Box::new(device.to_owned());
            dyn_nodes.insert(*node_id, dyn_node);
        }
        return dyn_nodes;
    }

    fn build_pool_impl(&self, nodes: HashMap<NodeId, Box<dyn Node>>) -> PoolImpl {
        let mut power_schedule_map: HashMap<NodeId, PowerSchedule> = HashMap::new();
        for (_, schedule_map) in self.power_schedules.iter() {
            for (node_id, schedule) in schedule_map.iter() {
                power_schedule_map.insert(*node_id, schedule.to_owned());
            }
        }
        let node_pool = PoolImpl::new(nodes, power_schedule_map);
        return node_pool;
    }

    pub fn get_duration(&self) -> u64 {
        return self.base_config.simulation_settings.sim_duration;
    }

    pub fn get_time_step(&self) -> u64 {
        return self.base_config.simulation_settings.sim_step;
    }
}
