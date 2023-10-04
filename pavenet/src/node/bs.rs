use core::fmt;
use std::hash::{Hash, Hasher};

use pavenet_device::device::common::{Status, Timing};
use crate::models::composer::bs::{BSComposer, BasicBSComposer};
use pavenet::models::composer::{BasicComposer, ComposerType, StatusComposer};
use pavenet::models::responder::{ResponderType, StatsResponder};
use pavenet::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use log::debug;

use pavenet_core::::config::{BaseStationSettings, DeviceSettings, DeviceType};
use pavenet_core::::dyn_config::{EpisodeInfo, EpisodeType};
use crate::sim::core::Core;

#[derive(Clone, Copy)]
pub struct BaseStation {
    pub id: DeviceId,
    storage: f32,
    pub location: Real2D,
    pub bs_info: BSInfo,
    pub timing: Timing,
    pub composer: BSComposer,
    pub responder: ResponderType,
    pub status: Status,
    pub device_type: DeviceType,
    pub device_class: u32,
    step: TimeStamp,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct BSInfo {
    pub location: Real2D,
    pub temperature: f32,
    pub storage: f32,
}

impl BaseStation {
    pub fn new(id: u64, timing_info: Timing, bs_settings: &DeviceSettings) -> Self {
        let composer: Option<ComposerType> = match bs_settings.composer {
            Some(ref composer_settings) => match composer_settings.name.as_str() {
                "basic" => Some(ComposerType::Basic(BasicComposer::new(composer_settings))),
                "status" => Some(ComposerType::Status(StatusComposer::new(composer_settings))),
                _ => panic!("Unknown composer type"),
            },
            None => None,
        };

        let simplifier: Option<SimplifierType> = match bs_settings.simplifier {
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

        let responder: ResponderType = match bs_settings.responder.name.as_str() {
            _ => ResponderType::Stats(StatsResponder::new()),
        };
        Self {
            id,
            timing: timing_info,
            storage: bs_settings.storage,
            device_class: bs_settings.device_class,
            composer,
            responder,
            location: Real2D::default(),
            bs_info: BSInfo::default(),
            step: TimeStamp::default(),
            status: Status::Inactive,
            device_type: DeviceType::BaseStation,
        }
    }

    fn update_geo_data(&mut self, core_state: &mut Core) {
        match core_state.device_field.position_cache.remove(&self.id) {
            Some(loc) => {
                self.location = loc;
                core_state
                    .device_field
                    .bs_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }
    }

    pub fn deactivate(&mut self, core_state: &mut Core) {
        self.status = Status::Inactive;
        self.timing.increment_timing_index();
        core_state.devices_to_pop.base_stations.push(self.id);

        let time_stamp = self.timing.pop_activation_time();
        if time_stamp > self.step {
            core_state
                .devices_to_add
                .base_stations
                .push((self.id, time_stamp));
        }
    }

    pub fn forward_device_data_to_controller(&mut self, core_state: &mut Core) {
        // Collect vehicle and RSU data
        let vehicles_data = match core_state.vanet.uplink.v2bs_data.remove(&self.id) {
            Some(bs_data) => bs_data,
            None => vec![],
        };
        let rsu_data = match core_state.vanet.uplink.rsu2bs_data.remove(&self.id) {
            Some(rsu_data) => rsu_data,
            None => vec![],
        };

        // Send responses to vehicles
        let bs_responses = match self.responder {
            ResponderType::Stats(responder) => {
                responder.respond_to_vehicles(&vehicles_data, rsu_data.len())
            }
        };
        core_state
            .vanet
            .downlink
            .bs2v_responses
            .extend(bs_responses.into_iter());

        // Compose BS payload and send to controller
        let mut bs_payload = match self.composer {
            BSComposer::Basic(composer) => composer.compose_bs2c_payload(vehicles_data, rsu_data),
        };
        bs_payload.id = self.id;
        bs_payload.bs_info = self.bs_info;

        let controller_id = match core_state.vanet.infra_links.bs2c_links.get(&self.id) {
            Some(controller_id) => *controller_id,
            None => return,
        };

        core_state
            .vanet
            .uplink
            .bs2c_data
            .entry(controller_id)
            .and_modify(|v| v.push(bs_payload.clone()))
            .or_insert(vec![bs_payload]);
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

    fn update_models(&mut self, episode: &EpisodeInfo, reset_ts: TimeStamp) {}
}

impl Agent for BaseStation {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        self.status = Status::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);
        self.forward_device_data_to_controller(core_state);

        // Initiate deactivation if it is time
        if self.step == self.timing.peek_deactivation_time() {
            self.deactivate(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        false
    }
}

impl Hash for BaseStation {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for BaseStation {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, loc: Real2D) {
        self.location = loc;
    }
}

impl fmt::Display for BaseStation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for BaseStation {}

impl PartialEq for BaseStation {
    fn eq(&self, other: &BaseStation) -> bool {
        self.id == other.id
    }
}
