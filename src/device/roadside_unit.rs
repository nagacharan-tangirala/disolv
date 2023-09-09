use core::fmt;
use std::hash::{Hash, Hasher};

use crate::device::device_state::{DeviceState, Timing};
use crate::models::composer::{
    BasicComposer, ComposerType, DevicePayload, RandomComposer, SensorData,
};
use crate::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::hashbrown::HashMap;
use log::debug;

use crate::sim::core::Core;
use crate::utils::config::RSUSettings;
use crate::utils::constants::ARRAY_SIZE;
use crate::utils::ds_config::{DataSourceSettings, DeviceType, SensorType};

#[derive(Debug, Clone, Copy)]
pub struct RoadsideUnit {
    pub(crate) id: DeviceId,
    storage: f32,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) sensor_data: SensorData,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
    pub(crate) status: DeviceState,
    step: TimeStamp,
}

impl RoadsideUnit {
    pub(crate) fn new(
        id: u64,
        timing_info: Timing,
        rsu_settings: &RSUSettings,
        data_sources: [Option<DataSourceSettings>; ARRAY_SIZE],
    ) -> Self {
        let composer: ComposerType = match rsu_settings.composer.name.as_str() {
            "random" => ComposerType::Random(RandomComposer {
                data_sources: data_sources.clone(),
            }),
            _ => ComposerType::Basic(BasicComposer {
                data_sources: data_sources.clone(),
            }),
        };
        let simplifier: SimplifierType = match rsu_settings.composer.name.as_str() {
            "random" => SimplifierType::Random(RandomSimplifier {}),
            _ => SimplifierType::Basic(BasicSimplifier::new(rsu_settings.simplifier.clone())),
        };

        Self {
            id,
            storage: rsu_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            sensor_data: SensorData::default(),
            composer,
            simplifier,
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

    pub(crate) fn deactivate(&mut self, core_state: &mut Core) {
        self.status = DeviceState::Inactive;
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

    pub(crate) fn transfer_data_to_vehicles(&mut self, core_state: &mut Core) {
        let mut rsu2v_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::Vehicle),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };

        rsu2v_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2v_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2v_payload),
        };

        rsu2v_payload.id = self.id;
        rsu2v_payload.sensor_data = self.sensor_data;

        core_state
            .vanet
            .payloads
            .rsu2v_data
            .insert(self.id, rsu2v_payload);
    }

    pub(crate) fn transfer_data_to_bs(&mut self, core_state: &mut Core) {
        let mut rsu2bs_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::BaseStation),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        rsu2bs_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2bs_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2bs_payload),
        };
        rsu2bs_payload.id = self.id;
        rsu2bs_payload.sensor_data = self.sensor_data;

        core_state
            .vanet
            .payloads
            .rsu2bs_data
            .insert(self.id, rsu2bs_payload);
    }

    pub(crate) fn transfer_data_to_rsu(&mut self, core_state: &mut Core) {
        let mut rsu2rsu_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::RSU),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        rsu2rsu_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(rsu2rsu_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(rsu2rsu_payload),
        };
        rsu2rsu_payload.id = self.id;
        rsu2rsu_payload.sensor_data = self.sensor_data;

        core_state
            .vanet
            .payloads
            .rsu2rsu_data
            .insert(self.id, rsu2rsu_payload);
    }
}

impl Agent for RoadsideUnit {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        self.status = DeviceState::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);
        self.transfer_data_to_rsu(core_state);
        self.transfer_data_to_bs(core_state);
        self.transfer_data_to_vehicles(core_state);

        // Initiate deactivation if it is time
        if self.step == self.timing.peek_deactivation_time() {
            self.deactivate(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        false
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
