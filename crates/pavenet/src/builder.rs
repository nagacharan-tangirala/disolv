use crate::base::{BaseConfig, BaseConfigReader, NodeClassSettings, NodeSettings};
use crate::logger;
use krabmaga::rand_pcg::Pcg64Mcg;
use log::{debug, info};
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder, NodeType};
use pavenet_core::power::PowerManager;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::engine::GEngine;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::metrics::Measurable;
use pavenet_engine::node::{GNode, NodeId};
use pavenet_input::links::data::LinkReader;
use pavenet_input::power::data::{read_power_schedule, PowerTimes};
use pavenet_models::compose::Composer;
use pavenet_models::latency::LatencyType;
use pavenet_models::model::Model;
use pavenet_models::radio::{Radio, RadioModels, SlRadio};
use pavenet_models::reply::Replier;
use pavenet_models::select::Selector;
use pavenet_node::bucket::{DNodeScheduler, DeviceBucket};
use pavenet_node::device::{Device, DeviceModel};
use pavenet_node::linker::{Linker, LinkerSettings};
use pavenet_node::space::{Mapper, Space};
use pavenet_output::result::ResultWriter;
use std::path::{Path, PathBuf};

pub type DNode = GNode<DeviceBucket, Device, NodeOrder>;
pub type DEngine = GEngine<DeviceBucket>;

pub struct PavenetBuilder {
    base_config: BaseConfig,
    config_path: PathBuf,
}

impl PavenetBuilder {
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

    pub(crate) fn build(&mut self) -> DEngine {
        logger::initiate_logger(&self.config_path, &self.base_config.log_settings);

        info!("Building devices and device pools...");
        let device_map = self.build_devices();
        let node_map = self.build_nodes(device_map);

        let scheduler = DNodeScheduler::new(node_map);
        let bucket = self.build_device_bucket(scheduler);

        info!("Initializing Engine...");
        DEngine::builder()
            .bucket(bucket)
            .end_step(self.duration())
            .streaming_interval(self.streaming_step())
            .step_size(self.step_size())
            .build()
    }

    fn read_power_schedules(&self, node_type: NodeType) -> HashMap<NodeId, PowerTimes> {
        let node_settings: &NodeSettings = self
            .base_config
            .nodes
            .iter()
            .find(|x| x.node_type == node_type)
            .unwrap_or_else(|| panic!("Invalid node type: {}", node_type));

        let power_file = Path::new(&self.config_path).join(&node_settings.power_file);
        Self::read_power_file(&power_file)
    }

    fn read_power_file(power_file: &PathBuf) -> HashMap<NodeId, PowerTimes> {
        if !power_file.exists() {
            panic!("Power schedule file {} is not found.", power_file.display());
        }
        return match read_power_schedule(&power_file) {
            Ok(power_schedule) => power_schedule,
            Err(e) => {
                panic!("Error while parsing the power schedule file: {}", e);
            }
        };
    }

    fn read_node_ids(power_schedules: &HashMap<NodeId, PowerTimes>) -> Vec<NodeId> {
        let mut node_ids: Vec<NodeId> = Vec::new();
        power_schedules.iter().for_each(|(node_id, _)| {
            node_ids.push(*node_id);
        });
        node_ids
    }

    fn build_devices(&mut self) -> HashMap<NodeId, Device> {
        info!("Building devices...");
        let mut device_map = HashMap::new();

        for node_setting in self.base_config.nodes.clone().iter() {
            let mut power_schedules = self.read_power_schedules(node_setting.node_type);
            let node_ids = Self::read_node_ids(&power_schedules);
            let node_count = node_ids.len();
            info!("Building devices for node type: {}", node_setting.node_type);

            for class_settings in node_setting.class.iter() {
                let class_count = (class_settings.node_share * node_count as f32) as usize;
                let mut device_count = 0;

                for node_id in node_ids.iter() {
                    let node_schedule = power_schedules
                        .remove(node_id)
                        .unwrap_or_else(|| panic!("Invalid node id"));
                    let device = self.build_device(
                        *node_id,
                        &node_setting.node_type,
                        class_settings,
                        node_schedule,
                    );
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
        &mut self,
        node_id: NodeId,
        node_type: &NodeType,
        class_settings: &NodeClassSettings,
        power_times: PowerTimes,
    ) -> Device {
        let node_info = Self::build_node_info(node_id, node_type, class_settings);

        let mut radio = self.build_radio(class_settings);
        radio.update_settings(&class_settings.tx_actions);
        let mut sl_radio = self.build_sl_radio(class_settings);
        sl_radio.update_settings(&class_settings.sl_actions);

        let target_classes = self.read_target_classes(class_settings);
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
            .radio(radio)
            .sl_radio(sl_radio)
            .composer(Composer::with_settings(&class_settings.composer))
            .selector(selector_vec)
            .replier(Replier::with_settings(&class_settings.replier))
            .build();

        let device = Device::builder()
            .node_info(node_info)
            .models(device_model)
            .target_classes(target_classes)
            .build();

        return device;
    }

    fn build_node_info(
        node_id: NodeId,
        node_type: &NodeType,
        class_settings: &NodeClassSettings,
    ) -> NodeInfo {
        NodeInfo::builder()
            .id(node_id)
            .node_type(node_type.to_owned())
            .node_class(class_settings.node_class)
            .node_order(class_settings.node_order)
            .build()
    }

    fn build_radio(&self, class_settings: &NodeClassSettings) -> Radio {
        let radio_models = RadioModels::builder()
            .latency_type(LatencyType::with_settings(
                class_settings.rx.latency.clone(),
            ))
            .build();

        Radio::builder()
            .models(radio_models)
            .step_size(TimeMS::from(self.sim_step()))
            .rng(Pcg64Mcg::new(self.sim_seed()))
            .build()
    }

    fn build_sl_radio(&self, class_settings: &NodeClassSettings) -> SlRadio {
        let radio_models = RadioModels::builder()
            .latency_type(LatencyType::with_settings(
                class_settings.sl.latency.clone(),
            ))
            .build();

        SlRadio::builder()
            .models(radio_models)
            .step_size(TimeMS::from(self.sim_step()))
            .rng(Pcg64Mcg::new(self.sim_seed()))
            .build()
    }

    fn read_target_classes(&self, class_settings: &NodeClassSettings) -> Vec<NodeClass> {
        class_settings
            .composer
            .source_settings
            .iter()
            .filter(|x| x.node_class != class_settings.node_class)
            .map(|x| x.node_class)
            .collect()
    }

    fn build_nodes(&self, devices: HashMap<NodeId, Device>) -> HashMap<NodeId, DNode> {
        let mut node_map = HashMap::with_capacity(devices.len());
        for (node_id, device) in devices.iter() {
            let node = DNode::new(*node_id, device.to_owned());
            node_map.insert(*node_id, node);
        }
        node_map
    }

    fn build_device_bucket(&mut self, scheduler: DNodeScheduler) -> DeviceBucket {
        DeviceBucket::builder()
            .scheduler(scheduler)
            .mapper_holder(self.build_mapper_vec())
            .linker_holder(self.build_linker_vec())
            .space(self.build_space())
            .class_to_type(self.read_class_to_type_map())
            .output_step(self.output_step())
            .resultant(ResultWriter::new(&self.base_config.output_settings))
            .build()
    }

    fn build_mapper_vec(&self) -> Vec<(NodeType, Mapper)> {
        let mut mapper_vec: Vec<(NodeType, Mapper)> = Vec::new();
        for node_setting in self.base_config.nodes.iter() {
            let node_type = node_setting.node_type;
            let mapper = Mapper::builder(&self.config_path)
                .streaming_step(self.streaming_step())
                .field_settings(self.base_config.field_settings.clone())
                .space_settings(node_setting.mobility.clone())
                .build();
            mapper_vec.push((node_type, mapper));
        }
        mapper_vec
    }

    fn build_linker_vec(&self) -> Vec<Linker> {
        let mut linker_vec: Vec<Linker> = Vec::new();
        for node_setting in self.base_config.nodes.iter() {
            let node_type = node_setting.node_type;
            match node_setting.linker {
                Some(ref linker_settings) => {
                    for link_setting in linker_settings.iter() {
                        let linker = self.build_linker(&node_type, link_setting);
                        linker_vec.push(linker);
                    }
                }
                None => {}
            };
        }
        linker_vec
    }

    fn build_linker(&self, source_type: &NodeType, link_config: &LinkerSettings) -> Linker {
        let links_file = self.config_path.join(&link_config.links_file);
        if !links_file.exists() {
            panic!("Link file {} is not found.", links_file.display());
        }
        let link_reader =
            LinkReader::new(links_file, self.streaming_step(), link_config.is_streaming);
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

    fn read_class_to_type_map(&mut self) -> HashMap<NodeClass, NodeType> {
        let mut class_to_type: HashMap<NodeClass, NodeType> = HashMap::new();
        for node_setting in self.base_config.nodes.iter() {
            let node_classes: Vec<NodeClass> =
                node_setting.class.iter().map(|x| x.node_class).collect();
            for node_class in node_classes.iter() {
                class_to_type.insert(node_class.to_owned(), node_setting.node_type);
            }
        }
        class_to_type
    }

    fn streaming_step(&self) -> TimeMS {
        return self.base_config.simulation_settings.sim_streaming_step;
    }

    pub(crate) fn duration(&self) -> TimeMS {
        return self.base_config.simulation_settings.sim_duration;
    }

    fn step_size(&self) -> TimeMS {
        return self.base_config.simulation_settings.sim_step_size;
    }

    fn sim_step(&self) -> TimeMS {
        return self.base_config.simulation_settings.sim_step_size;
    }

    fn sim_seed(&self) -> u128 {
        return u128::from(self.base_config.simulation_settings.sim_seed);
    }

    fn output_step(&self) -> TimeMS {
        return self.base_config.output_settings.output_step;
    }
}
