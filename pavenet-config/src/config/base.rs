use crate::config::types::{DeviceId, TimeStamp};
use hashbrown::HashMap;
use serde_derive::Deserialize;
use std::path::PathBuf;

pub type Trace = (Vec<DeviceId>, Vec<f32>, Vec<f32>, Vec<f32>); // (device_id, x, y, velocity)
pub type TraceMap = HashMap<TimeStamp, Option<Trace>>;
pub type Activation = (Vec<TimeStamp>, Vec<TimeStamp>); // (start_time, end_time)
pub type MultiLink = (Vec<DeviceId>, Vec<f32>); // (neighbour_device_ids, distances)
pub type MultiLinkMap = HashMap<DeviceId, MultiLink>;
pub type LinkMap = HashMap<DeviceId, DeviceId>; // (source_id, target_id)
pub type TraceLinkMap = HashMap<TimeStamp, MultiLinkMap>;

#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
pub enum LinkType {
    #[default]
    Single,
    Multiple,
}

#[derive(Deserialize, Debug, Clone)]
pub enum MobilityType {
    Stationary,
    Mobile,
}

#[derive(Deserialize, Clone, Debug, Copy)]
pub enum TransferMode {
    UDT,
    BDT,
}

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Vehicle = 0,
    RSU,
    BaseStation,
    Controller,
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

#[derive(Deserialize, Debug, Clone)]
pub struct BaseConfig {
    pub simulation_settings: SimSettings,
    pub log_settings: LogSettings,
    pub output_settings: OutputSettings,
    pub field_settings: FieldSettings,
    pub devices: Vec<DeviceSettings>,
    pub network_settings: NetworkSettings,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSettings {
    pub ratio: f32,
    pub device_type: DeviceType,
    pub device_class: u32,
    pub hierarchy: u32,
    pub activation_file: String,
    pub mobility_settings: MobilitySettings,
    pub linker: LinkerSettings,
    pub composer: Option<ComposerSettings>,
    pub simplifier: Option<SimplifierSettings>,
    pub responder: Option<ResponderSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimSettings {
    pub sim_name: String,
    pub sim_duration: u64,
    pub sim_step: u64,
    pub sim_streaming_step: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogSettings {
    pub log_path: String,
    pub log_level: String,
    pub log_file_name: String,
    pub log_overwrite: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutputSettings {
    pub output_path: String,
    pub output_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<SourceSettings>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct SourceSettings {
    pub data_type: DataType,
    pub data_count: u32,
    pub unit_size: f32,
    pub frequency: u32,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct SimplifierSettings {
    pub name: String,
    pub compression_factor: Option<f32>,
    pub sampling_factor: Option<f32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MobilitySettings {
    pub mobility_type: MobilityType,
    pub geo_data_file: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FieldSettings {
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSettings {}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkerSettings {
    pub name: String,
    pub links: Vec<LinkConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinkConfig {
    pub link_type: LinkType,
    pub transfer_mode: TransferMode,
    pub target_device: DeviceType,
    pub range: f32,
    pub links_file: String,
}
