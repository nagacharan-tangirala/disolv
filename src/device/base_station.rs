use core::fmt;
use std::hash::{Hash, Hasher};

use crate::device::device_state::{DeviceState, Timing};
use crate::models::aggregator::{AggregatorType, BasicAggregator};
use crate::models::composer::UplinkPayload;
use crate::models::responder::{ResponderType, StatsResponder};
use crate::reader::activation::{DeviceId, TimeStamp};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::hashbrown::HashMap;
use log::debug;

use crate::sim::core::Core;
use crate::utils::config::BaseStationSettings;

#[derive(Clone, Copy)]
pub(crate) struct BaseStation {
    pub(crate) id: DeviceId,
    storage: f32,
    pub(crate) location: Real2D,
    pub(crate) bs_info: BSInfo,
    pub(crate) timing: Timing,
    pub(crate) aggregator: AggregatorType,
    pub(crate) responder: ResponderType,
    pub(crate) status: DeviceState,
    step: TimeStamp,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct BSInfo {
    pub(crate) location: Real2D,
    pub(crate) temperature: f32,
    pub(crate) storage: f32,
}

impl BaseStation {
    pub(crate) fn new(id: u64, timing_info: Timing, bs_settings: &BaseStationSettings) -> Self {
        let aggregator: AggregatorType = match bs_settings.aggregator.name.as_str() {
            _ => AggregatorType::Basic(BasicAggregator::new()),
        };
        let responder: ResponderType = match bs_settings.responder.name.as_str() {
            _ => ResponderType::Stats(StatsResponder::new()),
        };
        Self {
            id,
            storage: bs_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            bs_info: BSInfo::default(),
            aggregator,
            responder,
            status: DeviceState::Inactive,
            step: 0,
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

    pub(crate) fn deactivate(&mut self, core_state: &mut Core) {
        self.status = DeviceState::Inactive;
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

    pub(crate) fn forward_device_data_to_controller(&mut self, core_state: &mut Core) {
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
        let mut bs_responses = match self.responder {
            ResponderType::Stats(responder) => {
                responder.respond_to_vehicles(&vehicles_data, rsu_data.len())
            }
        };
        core_state
            .vanet
            .downlink
            .bs2v_responses
            .extend(bs_responses.drain());

        // Aggregate and forward data to controller
        let mut bs_payload = match self.aggregator {
            AggregatorType::Basic(aggregator) => aggregator.aggregate(vehicles_data, rsu_data),
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
            .insert(controller_id, bs_payload);
    }
}

impl Agent for BaseStation {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        self.status = DeviceState::Active;
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
