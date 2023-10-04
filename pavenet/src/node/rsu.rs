use core::fmt;
use std::hash::{Hash, Hasher};

use pavenet_core::::config::{DeviceSettings, DeviceType};
use pavenet_core::::dyn_config::{EpisodeInfo, EpisodeType};
use pavenet_device::device::common::{Status, Timing};
use pavenet::models::composer::{BasicComposer, ComposerType, SensorData, StatusComposer};
use pavenet::models::links::rsu::{RSULinkerType, SimpleRSULinker};
use pavenet::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use crate::sim::core::Core;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use log::debug;
use pavenet_config::config::base::NodeType;
use pavenet_config::config::types::{DeviceId, TimeStamp};
use pavenet_models::device::composer::SensorData;
use pavenet_models::device::mapstate::MapState;
use pavenet_models::device::power::PowerState;
use crate::devices::model::DeviceModel;
use crate::devices::vehicle::DataStats;

#[derive(Debug, Clone, Copy)]
pub struct RoadsideUnit {
    pub id: DeviceId,
    pub device_type: NodeType,
    pub device_class: u32,
    pub hierarchy: i32,
    pub data_stats: DataStats,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub sensor_data: SensorData,
    pub models: DeviceModel,
    step: TimeStamp,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct RSUDataStats {
    pub generated_data_size: f32,
    pub outgoing_rsu2bs_data_size: f32,
    pub outgoing_rsu2rsu_data_size: f32,
    pub outgoing_rsu2v_data_size: f32,
    pub assigned_bs_id: Option<DeviceId>,
    pub rsu2bs_latency_factor: f32,
    pub rsu2rsu_latency_factor: f32,
    pub rsu2v_latency_factor: f32,
    pub incoming_rsu2rsu_data_size: f32,
    pub incoming_v2rsu_data_size: f32,
}

impl RoadsideUnit {
    pub fn new(id: u64, timing_info: Timing, rsu_settings: &DeviceSettings) -> Self {
        let composer: Option<ComposerType> = match rsu_settings.composer {
            Some(ref composer_settings) => match composer_settings.name.as_str() {
                "basic" => Some(ComposerType::Basic(BasicComposer::new(composer_settings))),
                "status" => Some(ComposerType::Status(StatusComposer::new(composer_settings))),
                _ => panic!("Unknown composer type"),
            },
            None => None,
        };

        let simplifier: Option<SimplifierType> = match rsu_settings.simplifier {
            Some(ref simplifier_settings) => match simplifier_settings.name.as_str() {
                "basic" => Some(SimplifierType::Basic(BasicSimplifier::new(
                    simplifier_settings,
                ))),
                "random" => Some(SimplifierType::Random(RandomSimplifier::new(
                    simplifier_settings,
                ))),
                _ => panic!("Unknown simplifier type"),
            },
            None => None,
        };

        let linker: RSULinkerType = match rsu_settings.linker.name.as_str() {
            "simple" => RSULinkerType::Simple(SimpleRSULinker::new(rsu_settings.linker.clone())),
            _ => RSULinkerType::Simple(SimpleRSULinker::new(rsu_settings.linker.clone())),
        };

        Self {
            id,
            timing: timing_info,
            storage: rsu_settings.storage,
            device_class: rsu_settings.device_class,
            linker,
            composer,
            simplifier,
            location: Real2D::default(),
            sensor_data: SensorData::default(),
            rsu_data_stats: RSUDataStats::default(),
            step: TimeStamp::default(),
            status: Status::Inactive,
            device_type: NodeType::RSU,
        }
    }

    fn update_geo_data(&mut self, core_state: &mut Core) {
        match core_state.device_field.position_cache.remove(&self.id) {
            Some(loc) => {
                self.location = loc;
                core_state
                    .device_field
                    .rsu_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }
        self.sensor_data.speed = match core_state.device_field.velocity_cache.remove(&self.id) {
            Some(speed) => speed,
            None => 0.0,
        };
    }

    pub fn deactivate(&mut self, core_state: &mut Core) {
        self.status = Status::Inactive;
        self.timing.increment_timing_index();
        core_state.devices_to_pop.roadside_units.push(self.id);

        let time_stamp = self.timing.pop_activation_time();
        if time_stamp > self.step {
            core_state
                .devices_to_add
                .roadside_units
                .push((self.id, time_stamp));
        }
    }

    pub fn transfer_data_to_vehicles(&mut self, core_state: &mut Core) {
        let mut rsu2v_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(NodeType::Vehicle),
            ComposerType::Status(ref composer) => composer.compose_payload(),
        };

        rsu2v_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2v_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2v_payload),
        };

        rsu2v_payload.id = self.id;
        rsu2v_payload.sensor_data = self.sensor_data;

        let rsu2v_links: Vec<DeviceId> = match core_state
            .vanet
            .mesh_links
            .rsu2v_link_cache
            .remove(&self.id)
        {
            Some(rsu2v_links) => rsu2v_links,
            None => Vec::new(),
        };

        for vehicle_id in rsu2v_links {
            core_state
                .vanet
                .uplink
                .rsu2v_data
                .entry(vehicle_id)
                .and_modify(|payload| payload.push(rsu2v_payload.clone()))
                .or_insert(vec![rsu2v_payload.clone()]);
        }
    }

    pub fn transfer_data_to_bs(&mut self, core_state: &mut Core) {
        let mut rsu2bs_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(NodeType::BaseStation),
            ComposerType::Status(ref composer) => composer.compose_payload(),
        };
        rsu2bs_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2bs_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2bs_payload),
        };
        rsu2bs_payload.id = self.id;
        rsu2bs_payload.sensor_data = self.sensor_data;

        let rsu2bs_links = core_state
            .vanet
            .infra_links
            .rsu2bs_link_cache
            .remove(&self.id);

        let selected_bs_id = match self.linker {
            RSULinkerType::Simple(ref linker) => linker.find_bs_link(rsu2bs_links),
        };
        self.rsu_data_stats.assigned_bs_id = selected_bs_id;

        match selected_bs_id {
            Some(bs_id) => {
                core_state
                    .vanet
                    .uplink
                    .rsu2bs_data
                    .entry(bs_id)
                    .and_modify(|payload| payload.push(rsu2bs_payload.clone()))
                    .or_insert(vec![rsu2bs_payload.clone()]);
            }
            None => {}
        }
    }

    pub fn transfer_data_to_rsu(&mut self, core_state: &mut Core) {
        let mut rsu2rsu_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(NodeType::RSU),
            ComposerType::Status(ref composer) => composer.compose_payload(),
        };
        rsu2rsu_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2rsu_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2rsu_payload),
        };
        rsu2rsu_payload.id = self.id;
        rsu2rsu_payload.sensor_data = self.sensor_data;

        let rsu2rsu_links = core_state
            .vanet
            .mesh_links
            .rsu2rsu_link_cache
            .remove(&self.id);

        let selected_rsu_ids = match self.linker {
            RSULinkerType::Simple(ref linker) => linker.find_rsu_mesh_links(rsu2rsu_links),
        };

        for rsu_id in selected_rsu_ids.unwrap_or(Vec::new()) {
            core_state
                .vanet
                .uplink
                .rsu2rsu_data
                .entry(rsu_id)
                .and_modify(|payload| payload.push(rsu2rsu_payload.clone()))
                .or_insert(vec![rsu2rsu_payload.clone()]);
        }
    }

    pub fn setup_episode(&mut self, episode: EpisodeInfo) {
        match episode.episode_type {
            EpisodeType::Temporary => {
                let duration = match episode.duration {
                    Some(duration) => duration,
                    None => panic!("Duration must be specified for temporary episodes."),
                };
                self.update_models(&episode, self.step + duration);
            }
            EpisodeType::Persistent => self.update_models(&episode, 0),
        }
    }

    fn update_models(&mut self, episode: &EpisodeInfo, reset_ts: TimeStamp) {
        if let Some(data_sources) = &episode.data_sources {
            match &mut self.composer {
                ComposerType::Basic(ref mut composer) => {
                    composer
                        .settings_handler
                        .update_settings(data_sources, reset_ts);
                }
                ComposerType::Status(ref mut composer) => {
                    composer
                        .settings_handler
                        .update_settings(data_sources, reset_ts);
                }
            }
        }

        if let Some(simplifier_settings) = &episode.simplifier {
            match &mut self.simplifier {
                SimplifierType::Basic(ref mut simplifier) => {
                    simplifier
                        .settings
                        .update_settings(simplifier_settings, reset_ts);
                }
                SimplifierType::Random(ref mut simplifier) => {
                    simplifier
                        .settings
                        .update_settings(simplifier_settings, reset_ts);
                }
            }
        }

        if let Some(linker_settings) = &episode.rsu_linker {
            match &mut self.linker {
                RSULinkerType::Simple(ref mut linker) => {
                    linker
                        .settings_handler
                        .update_settings(linker_settings, reset_ts);
                }
            }
        }
    }
}

impl Agent for RoadsideUnit {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is ON", self.id);
        self.power_state = PowerState::On;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;
        self.update_map_state(core_state);
        self.transmit_data(core_state);

        // Initiate deactivation if it is time
        if self.step == self.models.power_schedule.peek_deactivation_time() {
            self.power_off(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        debug!("{} is OFF", self.id);
        self.power_state == PowerState::Off
    }

    fn before_step(&mut self, state: &mut dyn State) -> () {
        debug!("Before step {}", self.step);
        self.before_stepping(state);
    }

    fn after_step(&mut self, state: &mut dyn State) -> () {
        debug!("After step {}", self.step);
        self.after_stepping(state);
    }
}

impl Hash for RoadsideUnit {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for RoadsideUnit {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, new_location: Real2D) {
        self.location = new_location;
    }
}

impl fmt::Display for RoadsideUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for RoadsideUnit {}

impl PartialEq for RoadsideUnit {
    fn eq(&self, other: &RoadsideUnit) -> bool {
        self.id == other.id
    }
}
