use std::path::{Path, PathBuf};

use burn::backend::{Autodiff, Wgpu};
use burn::backend::wgpu::WgpuDevice;
use hashbrown::HashMap;
use log::info;

use disolv_core::agent::{AgentClass, AgentId, AgentKind};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Measurable;
use disolv_core::model::Model;
use disolv_core::scheduler::DefaultScheduler;
use disolv_input::links::LinkReader;
use disolv_input::power::{PowerTimes, read_power_schedule};
use disolv_models::device::actor::Actor;
use disolv_models::device::directions::Directions;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::power::PowerManager;
use disolv_models::net::network::Network;
use disolv_output::logger::initiate_logger;
use disolv_output::result::Results;
use disolv_output::ui::SimUIMetadata;

use crate::fl::agent::FAgent::{FClient, FServer};
use crate::fl::bucket::{FlBucket, FlBucketModels, FlDataLake, FlNetwork};
use crate::fl::client::{Client, ClientModels};
use crate::fl::device::{Device, DeviceInfo, DeviceModels};
use crate::fl::server::{FlServerModels, Server};
use crate::models::ai::aggregate::Aggregator;
use crate::models::ai::compose::FlComposer;
use crate::models::ai::data::DataHolder;
use crate::models::ai::mnist::{MnistModelConfig, MnistTrainingConfig};
use crate::models::ai::models::ModelType;
use crate::models::ai::select::ClientSelector;
use crate::models::ai::times::{ClientTimes, ServerTimes};
use crate::models::ai::trainer::{Trainer, TrainerSettings};
use crate::models::device::energy::EnergyType;
use crate::models::device::hardware::Hardware;
use crate::models::device::lake::ModelLake;
use crate::models::device::link::LinkSelector;
use crate::models::device::linker::{Linker, LinkerSettings};
use crate::models::device::mapper::{GeoMap, GeoMapper};
use crate::models::device::network::{RadioMetrics, RadioResources, Slice, SliceSettings};
use crate::simulation::config::{
    AgentClassSettings, BaseConfig, BaseConfigReader, ClientClassSettings, ServerClassSettings,
};
use crate::simulation::distribute::DataDistributor;
use crate::simulation::ui::SimRenderer;

pub type FlBackend = Wgpu<f32, i32>;
pub type FlAdBackend = Autodiff<FlBackend>;

pub type FedDevice = Device<FlAdBackend>;
pub type FedBucket = FlBucket<FlAdBackend>;
pub type FedClient = Client<FlAdBackend>;
pub type FedServer = Server<FlAdBackend>;

pub type DScheduler = DefaultScheduler<FedDevice, FedBucket>;

pub struct SimulationBuilder {
    base_config: BaseConfig,
    config_path: PathBuf,
    metadata: SimUIMetadata,
    default_device: WgpuDevice,
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

        let config_reader = BaseConfigReader::new(base_config_file);
        match config_reader.parse() {
            Ok(base_config) => {
                let metadata = Self::build_metadata(&base_config, base_config_file);
                Self {
                    base_config,
                    config_path,
                    metadata,
                    default_device: WgpuDevice::BestAvailable,
                }
            }
            Err(e) => {
                panic!("Error while parsing the base configuration file: {}", e);
            }
        }
    }

    fn build_metadata(base_config: &BaseConfig, base_config_file: &str) -> SimUIMetadata {
        SimUIMetadata {
            scenario: base_config.simulation_settings.scenario.clone(),
            input_file: base_config_file.to_owned(),
            output_path: base_config.output_settings.output_path.clone(),
            log_path: base_config.log_settings.log_path.clone(),
        }
    }

    pub(crate) fn build(&mut self) -> DScheduler {
        initiate_logger(
            &self.config_path,
            &self.base_config.log_settings,
            Some(self.base_config.output_settings.scenario_id),
        );

        info!("Building devices and device pools...");
        let device_bucket = self.build_fl_bucket();
        let agent_map = self.build_agents();
        self.build_scheduler(agent_map, device_bucket)
    }

    fn build_agents(&mut self) -> HashMap<AgentId, FedDevice> {
        info!("Building clients...");
        let mut fed_agents = self.build_devices();
        info!("Building servers...");
        fed_agents.extend(self.build_servers());
        fed_agents
    }

    fn build_devices(&mut self) -> HashMap<AgentId, FedDevice> {
        let mut client_map = HashMap::new();

        for client_setting in self.base_config.clients.clone().iter() {
            let mut power_schedules = self.read_client_power_schedules(client_setting.agent_type);
            let client_ids = Self::read_device_ids(&power_schedules);
            let client_count = client_ids.len();
            info!(
                "Building clients for device type: {}",
                client_setting.agent_type
            );

            for client_settings in client_setting.class.iter() {
                let class_count =
                    (client_settings.class_settings.agent_share * client_count as f32) as usize;
                let mut client_count = 0;

                for client_id in client_ids.iter() {
                    let client_schedule = power_schedules
                        .remove(client_id)
                        .unwrap_or_else(|| panic!("Invalid client id"));
                    let device = self.build_client(
                        *client_id,
                        &client_setting.agent_type,
                        client_settings,
                        client_schedule,
                    );
                    client_map.insert(*client_id, device);
                    client_count += 1;
                    if client_count == class_count {
                        break;
                    }
                }
            }
        }
        client_map
    }

    fn build_servers(&mut self) -> HashMap<AgentId, FedDevice> {
        let mut server_map = HashMap::new();

        for server_setting in self.base_config.servers.clone().iter() {
            let mut power_schedules = self.read_server_power_schedules(server_setting.agent_type);
            let server_ids = Self::read_device_ids(&power_schedules);
            let server_count = server_ids.len();
            info!(
                "Building servers for device type: {}",
                server_setting.agent_type
            );

            for server_settings in server_setting.class.iter() {
                let class_count =
                    (server_settings.class_settings.agent_share * server_count as f32) as usize;
                let mut server_count = 0;

                for server_id in server_ids.iter() {
                    let server_schedule = power_schedules
                        .remove(server_id)
                        .unwrap_or_else(|| panic!("Invalid server id"));
                    let server = self.build_server(
                        *server_id,
                        &server_setting.agent_type,
                        server_settings,
                        server_schedule,
                    );
                    server_map.insert(*server_id, server);
                    server_count += 1;
                    if server_count == class_count {
                        break;
                    }
                }
            }
        }
        server_map
    }

    fn build_client(
        &mut self,
        device_id: AgentId,
        device_type: &AgentKind,
        client_settings: &ClientClassSettings,
        power_times: PowerTimes,
    ) -> FedDevice {
        let client_info =
            Self::build_agent_info(device_id, device_type, &client_settings.class_settings);

        let power_manager = PowerManager::builder()
            .on_times(power_times.0.into())
            .off_times(power_times.1.into())
            .array_idx(0)
            .build();

        let mut selector_vec = Vec::new();
        client_settings
            .class_settings
            .link_selector
            .iter()
            .for_each(|settings| {
                let selector = LinkSelector::with_settings(settings);
                selector_vec.push((settings.target_class, selector));
            });

        let trainer = self.build_trainer(&client_settings.trainer_settings);
        let client_models = ClientModels::builder()
            .holder(DataHolder::with_settings(&client_settings.data_holder))
            .times(ClientTimes::with_settings(&client_settings.durations))
            .trainer(trainer)
            .build();

        let fed_client = FClient(
            FedClient::builder()
                .client_info(client_info)
                .fl_models(client_models)
                .build(),
        );

        let device_models = DeviceModels::builder()
            .power(power_manager)
            .flow(FlowRegister::default())
            .sl_flow(FlowRegister::default())
            .composer(FlComposer::with_settings(&client_settings.fl_composer))
            .actor(Actor::new(&client_settings.class_settings.actions.clone()))
            .directions(Directions::new(&client_settings.class_settings.directions))
            .hardware(Hardware::with_settings(&client_settings.hardware))
            .energy(EnergyType::with_settings(
                &client_settings.class_settings.energy,
            ))
            .link_selector(selector_vec)
            .build();

        FedDevice::builder()
            .fl_agent(fed_client)
            .device_info(client_info)
            .models(device_models)
            .build()
    }

    fn build_server(
        &mut self,
        device_id: AgentId,
        device_type: &AgentKind,
        server_settings: &ServerClassSettings,
        power_times: PowerTimes,
    ) -> FedDevice {
        let server_info =
            Self::build_agent_info(device_id, device_type, &server_settings.class_settings);

        let power_manager = PowerManager::builder()
            .on_times(power_times.0.into())
            .off_times(power_times.1.into())
            .array_idx(0)
            .build();

        let mut selector_vec = Vec::new();
        server_settings
            .class_settings
            .link_selector
            .iter()
            .for_each(|settings| {
                let selector = LinkSelector::with_settings(settings);
                selector_vec.push((settings.target_class, selector));
            });

        let trainer = self.build_trainer(&server_settings.trainer_settings);

        let fl_models = FlServerModels::builder()
            .client_classes(server_settings.client_classes.clone())
            .trainer(trainer)
            .times(ServerTimes::with_settings(&server_settings.durations))
            .aggregator(Aggregator::with_settings(&server_settings.aggregation))
            .client_selector(ClientSelector::with_settings(
                &server_settings.client_selector,
            ))
            .holder(DataHolder::with_settings(&server_settings.data_holder))
            .build();

        let fed_server = FServer(
            FedServer::builder()
                .server_info(server_info)
                .fl_models(fl_models)
                .build(),
        );

        let server_models = DeviceModels::builder()
            .power(power_manager)
            .flow(FlowRegister::default())
            .sl_flow(FlowRegister::default())
            .composer(FlComposer::with_settings(&server_settings.fl_composer))
            .actor(Actor::new(&server_settings.class_settings.actions.clone()))
            .directions(Directions::new(&server_settings.class_settings.directions))
            .hardware(Hardware::with_settings(&server_settings.hardware))
            .energy(EnergyType::with_settings(
                &server_settings.class_settings.energy,
            ))
            .link_selector(selector_vec)
            .build();

        FedDevice::builder()
            .fl_agent(fed_server)
            .device_info(server_info)
            .models(server_models)
            .build()
    }

    fn build_trainer(&self, trainer_settings: &TrainerSettings) -> Trainer<FlAdBackend> {
        let output_path = self.config_path.clone().join("train");

        match trainer_settings.model_type.to_lowercase().as_str() {
            "mnist" => {
                let mnist_settings = trainer_settings
                    .mnist_config_settings
                    .as_ref()
                    .expect("mnist settings missing from the config file");

                let train_config = MnistTrainingConfig::with_settings(mnist_settings);
                let mnist_model_config =
                    MnistModelConfig::new(mnist_settings.num_classes, mnist_settings.hidden_size)
                        .with_drop_out(mnist_settings.drop_out);

                Trainer::builder()
                    .no_of_weights(trainer_settings.no_of_weights)
                    .output_path(output_path)
                    .model(ModelType::Mnist(
                        mnist_model_config.init(&self.default_device),
                    ))
                    .config(train_config)
                    .build()
            }
            _ => panic!("Only mnist model is supported"),
        }
    }

    fn read_client_power_schedules(&self, agent_kind: AgentKind) -> HashMap<AgentId, PowerTimes> {
        let device_settings = self
            .base_config
            .clients
            .iter()
            .find(|x| x.agent_type == agent_kind)
            .unwrap_or_else(|| panic!("Invalid client type: {}", agent_kind));

        let power_file = Path::new(&self.config_path).join(&device_settings.power_file);
        if !power_file.exists() {
            panic!("Power schedule file {} is not found.", power_file.display());
        }
        read_power_schedule(&power_file)
    }

    fn read_server_power_schedules(&self, agent_kind: AgentKind) -> HashMap<AgentId, PowerTimes> {
        let device_settings = self
            .base_config
            .servers
            .iter()
            .find(|x| x.agent_type == agent_kind)
            .unwrap_or_else(|| panic!("Invalid server type: {}", agent_kind));

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

    fn build_agent_info(
        device_id: AgentId,
        device_type: &AgentKind,
        class_settings: &AgentClassSettings,
    ) -> DeviceInfo {
        DeviceInfo::builder()
            .id(device_id)
            .agent_type(device_type.to_owned())
            .agent_class(class_settings.agent_class)
            .agent_order(class_settings.agent_order)
            .build()
    }

    fn build_scheduler(
        &mut self,
        agent_map: HashMap<AgentId, FedDevice>,
        fl_bucket: FedBucket,
    ) -> DScheduler {
        info!("Building scheduler...");
        DefaultScheduler::builder()
            .duration(self.duration())
            .step_size(self.step_size())
            .agents(agent_map)
            .streaming_interval(self.streaming_interval())
            .output_interval(self.output_interval())
            .bucket(fl_bucket)
            .build()
    }

    fn build_fl_bucket(&mut self) -> FedBucket {
        info!("Building FL bucket...");
        let bucket_models = FlBucketModels::builder()
            .results(Results::new(&self.base_config.output_settings))
            .network(self.build_network())
            .space(self.build_space())
            .mapper_holder(self.build_mapper_vec())
            .linker_holder(self.build_linker_vec())
            .stats_holder(HashMap::new())
            .agent_data(HashMap::new())
            .data_lake(FlDataLake::new())
            .model_lake(ModelLake::new())
            .data_distributor(DataDistributor::with_settings(
                &self.base_config.bucket_models.distributor,
            ))
            .device(self.default_device.clone())
            .build();

        FedBucket::builder()
            .models(bucket_models)
            .class_to_type(self.read_class_to_type_map())
            .build()
    }

    fn build_mapper_vec(&self) -> Vec<(AgentKind, GeoMapper)> {
        let mut mapper_vec: Vec<(AgentKind, GeoMapper)> = Vec::new();
        for device_setting in self.base_config.clients.iter() {
            let device_type = device_setting.agent_type;
            let mapper = GeoMapper::builder(&self.config_path)
                .streaming_step(self.streaming_interval())
                .field_settings(self.base_config.field_settings.clone())
                .space_settings(device_setting.mobility.clone())
                .build();
            mapper_vec.push((device_type, mapper));
        }
        for device_setting in self.base_config.servers.iter() {
            let device_type = device_setting.agent_type;
            let mapper = GeoMapper::builder(&self.config_path)
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
        for client_setting in self.base_config.clients.iter() {
            let device_type = client_setting.agent_type;
            if let Some(ref linker_settings) = client_setting.linker {
                for link_setting in linker_settings.iter() {
                    let linker = self.build_linker(&device_type, link_setting);
                    linker_vec.push(linker);
                }
            };
        }
        for server_setting in self.base_config.servers.iter() {
            let device_type = server_setting.agent_type;
            if let Some(ref linker_settings) = server_setting.linker {
                for link_setting in linker_settings.iter() {
                    let linker = self.build_linker(&device_type, link_setting);
                    linker_vec.push(linker);
                }
            };
        }
        linker_vec
    }

    fn build_linker(&self, source_type: &AgentKind, link_config: &LinkerSettings) -> Linker {
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

    fn build_space(&self) -> GeoMap {
        GeoMap::builder()
            .height(self.base_config.field_settings.height)
            .cell_size(self.base_config.field_settings.cell_size)
            .width(self.base_config.field_settings.width)
            .build()
    }

    fn read_class_to_type_map(&mut self) -> HashMap<AgentClass, AgentKind> {
        let mut class_to_type: HashMap<AgentClass, AgentKind> = HashMap::new();
        for device_setting in self.base_config.clients.iter() {
            let device_classes: Vec<AgentClass> = device_setting
                .class
                .iter()
                .map(|x| x.class_settings.agent_class)
                .collect();
            for device_class in device_classes.iter() {
                class_to_type.insert(device_class.to_owned(), device_setting.agent_type);
            }
        }
        for device_setting in self.base_config.servers.iter() {
            let device_classes: Vec<AgentClass> = device_setting
                .class
                .iter()
                .map(|x| x.class_settings.agent_class)
                .collect();
            for device_class in device_classes.iter() {
                class_to_type.insert(device_class.to_owned(), device_setting.agent_type);
            }
        }
        class_to_type
    }

    fn build_network(&self) -> FlNetwork {
        let mut slices = HashMap::new();
        for slice_setting in self.base_config.network_settings.slice.iter() {
            let slice = Slice::builder()
                .id(slice_setting.id)
                .step_size(self.step_size())
                .resources(self.build_network_resources(slice_setting))
                .metrics(self.build_network_metrics(slice_setting))
                .build();
            slices.insert(slice_setting.name, slice);
        }
        Network::builder().slices(slices).build()
    }

    fn build_network_resources(&self, _slice_settings: &SliceSettings) -> RadioResources {
        RadioResources::builder().build()
    }

    fn build_network_metrics(&self, _slice_settings: &SliceSettings) -> RadioMetrics {
        RadioMetrics::builder().build()
    }

    fn streaming_interval(&self) -> TimeMS {
        self.base_config.simulation_settings.streaming_interval
    }

    fn output_interval(&self) -> TimeMS {
        self.base_config.output_settings.output_interval
    }

    fn duration(&self) -> TimeMS {
        self.base_config.simulation_settings.duration
    }

    fn step_size(&self) -> TimeMS {
        self.base_config.simulation_settings.step_size
    }

    fn sim_seed(&self) -> u128 {
        u128::from(self.base_config.simulation_settings.seed)
    }

    pub(crate) fn metadata(&self) -> SimUIMetadata {
        self.metadata.clone()
    }

    pub(crate) fn renderer(&self) -> SimRenderer {
        SimRenderer::new()
    }
}
