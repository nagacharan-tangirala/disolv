use core::fmt;
use std::hash::{Hash, Hasher};

use crate::device::device_state::{DeviceState, Timing};
use crate::device::roadside_unit::RSUPayload;
use crate::device::vehicle::VehiclePayload;
use crate::models::aggregator::{AggregatorType, BasicAggregator};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::{toroidal_transform, Location2D};
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::hashbrown::HashMap;
use krabmaga::rand;
use krabmaga::rand::Rng;

use crate::sim::core::Core;
use crate::utils::config::BaseStationSettings;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::DataSourceSettings;

/// The most basic agent should implement Clone, Copy and Agent to be able to be inserted in a Schedule.
#[derive(Clone, Copy)]
pub(crate) struct BaseStation {
    pub(crate) id: u64,
    storage: f32,
    pub(crate) location: Real2D,
    pub(crate) bs_info: BSInfo,
    pub(crate) timing: Timing,
    pub(crate) aggregator: AggregatorType,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BSPayload {
    pub(crate) id: u32,
    pub(crate) bs_info: BSInfo,
    pub(crate) v2bs_data: HashMap<u64, VehiclePayload>,
    pub(crate) rsu2bs_data: HashMap<u64, RSUPayload>,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct BSInfo {
    pub(crate) location: Real2D,
    pub(crate) temperature: f32,
    pub(crate) storage: f32,
}

impl BaseStation {
    pub(crate) fn new(id: u64, timing_info: Timing, bs_settings: &BaseStationSettings) -> Self {
        let data_sources: [Option<DataSourceSettings>; ARRAY_SIZE] = [None; ARRAY_SIZE];
        let aggregator: AggregatorType = match bs_settings.aggregator.name.as_str() {
            _ => AggregatorType::Basic(BasicAggregator {}),
        };
        Self {
            id,
            storage: bs_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            bs_info: BSInfo::default(),
            aggregator,
        }
    }
}

impl Agent for BaseStation {
    /// Put the code that should happen for each step, for each agent here.
    fn step(&mut self, state: &mut dyn State) {
        let state = state.as_any().downcast_ref::<Core>().unwrap();
        let mut rng = rand::thread_rng();

        // self.loc = Real2D { x: loc_x, y: loc_y };
        //
        // state
        //     .device_field
        //     .bs_field
        //     .set_object_location(*self, Real2D { x: loc_x, y: loc_y });
    }

    /// Put the code that decides if an agent should be removed or not
    /// for example in simulation where agents can die
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
