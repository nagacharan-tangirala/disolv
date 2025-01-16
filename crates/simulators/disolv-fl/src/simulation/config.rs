use std::path::PathBuf;

use serde::Deserialize;

use disolv_core::agent::{AgentClass, AgentKind, AgentOrder};
use disolv_core::bucket::TimeMS;
use disolv_models::device::directions::CommDirections;
use disolv_models::net::radio::ActionSettings;
use disolv_output::logger::LogSettings;
use disolv_output::result::OutputSettings;

use crate::models::ai::aggregate::AggregationSettings;
use crate::models::ai::compose::ComposerSettings;
use crate::models::ai::select::ClientSelectionSettings;
use crate::models::ai::times::ServerDurations;
use crate::models::ai::trainer::TrainerSettings;
use crate::models::data::allot::DataHolderSettings;
use crate::models::device::energy::EnergySettings;
use crate::models::device::hardware::HardwareSettings;
use crate::models::device::link::LinkSelectionSettings;
use crate::models::device::linker::LinkerSettings;
use crate::models::device::mapper::{FieldSettings, MobilitySettings};
use crate::models::device::message::MessageType;
use crate::models::device::network::SliceSettings;
use crate::simulation::distribute::DistributorSettings;

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub scenario: String,
    pub duration: TimeMS,
    pub step_size: TimeMS,
    pub streaming_interval: TimeMS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub slice: Vec<SliceSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentClassSettings {
    pub agent_share: f32,
    pub agent_class: AgentClass,
    pub agent_order: AgentOrder,
    pub link_selector: Vec<LinkSelectionSettings>,
    pub actions: Option<Vec<ActionSettings<MessageType>>>,
    pub directions: Vec<CommDirections>,
    pub energy: EnergySettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientClassSettings {
    pub fl_composer: ComposerSettings,
    pub hardware: HardwareSettings,
    pub data_holder: DataHolderSettings,
    pub trainer_settings: TrainerSettings,
    pub class_settings: AgentClassSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerClassSettings {
    pub client_classes: Vec<AgentClass>,
    pub client_selector: ClientSelectionSettings,
    pub fl_composer: ComposerSettings,
    pub aggregation: AggregationSettings,
    pub durations: ServerDurations,
    pub hardware: HardwareSettings,
    pub data_holder: DataHolderSettings,
    pub trainer_settings: TrainerSettings,
    pub class_settings: AgentClassSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientSettings {
    pub agent_type: AgentKind,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<ClientClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerSettings {
    pub agent_type: AgentKind,
    pub power_file: String,
    pub mobility: MobilitySettings,
    pub linker: Option<Vec<LinkerSettings>>,
    pub class: Vec<ServerClassSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BaseConfig {
    pub(crate) log_settings: LogSettings,
    pub(crate) simulation_settings: SimSettings,
    pub(crate) output_settings: OutputSettings,
    pub(crate) field_settings: FieldSettings,
    pub(crate) network_settings: NetworkSettings,
    pub(crate) clients: Vec<ClientSettings>,
    pub(crate) servers: Vec<ServerSettings>,
    pub(crate) bucket_models: BucketSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BucketSettings {
    pub distributor: DistributorSettings,
}

pub struct BaseConfigReader {
    file_path: PathBuf,
}

impl BaseConfigReader {
    pub fn new(file_name: &str) -> Self {
        let file_path = PathBuf::from(file_name);
        Self { file_path }
    }

    pub fn parse(&self) -> Result<BaseConfig, Box<dyn std::error::Error>> {
        let parsing_result = std::fs::read_to_string(&self.file_path)?;
        let config: BaseConfig = toml::from_str(&parsing_result)?;
        Ok(config)
    }
}
