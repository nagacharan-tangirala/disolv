use crate::device::device_state::{DeviceState, Timing};
use crate::models::composer::{
    BasicComposer, ComposerType, DataSources, RandomComposer, SensorData,
};
use crate::models::simplifier::{BasicSimplifier, RandomSimplifier, SimplifierType};
use crate::reader::activation::{DeviceId, TimeStamp};
use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::schedule::ScheduleOptions;
use krabmaga::engine::state::State;
use log::debug;
use std::hash::{Hash, Hasher};

use crate::models::links::veh_linker::{SimpleVehLinker, VehLinkerType};

use crate::sim::core::Core;
use crate::utils::config::DeviceType;
use crate::utils::config::VehicleSettings;
use crate::utils::constants::ARRAY_SIZE;

#[derive(Clone, Debug, Copy)]
pub(crate) struct Vehicle {
    pub(crate) id: DeviceId,
    pub(crate) location: Real2D,
    pub(crate) timing: Timing,
    pub(crate) sensor_data: SensorData,
    pub(crate) composer: ComposerType,
    pub(crate) simplifier: SimplifierType,
    pub(crate) linker: VehLinkerType,
    pub(crate) status: DeviceState,
    pub(crate) veh_data_stats: VDataStats,
    storage: f32,
    step: TimeStamp,
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct VDataStats {
    pub(crate) generated_data_size: f32,
    pub(crate) outgoing_v2bs_data_size: f32,
    pub(crate) outgoing_v2rsu_data_size: f32,
    pub(crate) outgoing_v2v_data_size: f32,
    pub(crate) assigned_bs_id: Option<DeviceId>,
    pub(crate) assigned_rsu_id: Option<DeviceId>,

    pub(crate) v2bs_latency_factor: f32,
    pub(crate) v2rsu_latency_factor: f32,
    pub(crate) v2v_latency_factor: f32,
    pub(crate) incoming_v2v_data_size: f32,
    pub(crate) incoming_rsu_data_size: f32,
}

impl Vehicle {
    pub(crate) fn new(
        id: u64,
        timing_info: Timing,
        vehicle_settings: &VehicleSettings,
        data_sources: [Option<DataSourceSettings>; ARRAY_SIZE],
    ) -> Self {
        let composer: ComposerType = match vehicle_settings.composer.name.as_str() {
            "random" => ComposerType::Random(RandomComposer::new(data_sources.clone())),
            _ => ComposerType::Basic(BasicComposer::new(data_sources.clone())),
        };
        let simplifier: SimplifierType = match vehicle_settings.simplifier.name.as_str() {
            "random" => SimplifierType::Random(RandomSimplifier {}),
            _ => SimplifierType::Basic(BasicSimplifier::new(vehicle_settings.simplifier.clone())),
        };
        let linker: VehLinkerType = match vehicle_settings.linker.name.as_str() {
            "simple" => {
                VehLinkerType::Simple(SimpleVehLinker::new(vehicle_settings.linker.clone()))
            }
            _ => VehLinkerType::Simple(SimpleVehLinker::new(vehicle_settings.linker.clone())),
        };

        Self {
            id,
            storage: vehicle_settings.storage,
            location: Real2D::default(),
            timing: timing_info,
            sensor_data: SensorData::default(),
            composer,
            simplifier,
            linker,
            status: DeviceState::Inactive,
            veh_data_stats: VDataStats::default(),
            step: 0,
        }
    }

    fn update_geo_data(&mut self, core_state: &mut Core) {
        match core_state.device_field.position_cache.remove(&self.id) {
            Some(loc) => {
                self.location = loc;
                core_state
                    .device_field
                    .vehicle_field
                    .set_object_location(*self, self.location);
            }
            None => {}
        }
        self.sensor_data.speed = match core_state.device_field.velocity_cache.remove(&self.id) {
            Some(speed) => speed,
            None => 0.0,
        };
    }

    fn deactivate(&mut self, core_state: &mut Core) {
        self.status = DeviceState::Inactive;
        self.timing.increment_timing_index();
        core_state.devices_to_pop.vehicles.push(self.id);

        let time_stamp = self.timing.pop_activation_time();
        if time_stamp > self.step {
            core_state
                .devices_to_add
                .vehicles
                .push((self.id, time_stamp));
        }
    }

    fn transfer_data_to_vehicles(&mut self, core_state: &mut Core) {
        let mut v2v_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::Vehicle),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };

        v2v_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2v_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2v_payload),
        };

        v2v_payload.id = self.id;
        v2v_payload.sensor_data = self.sensor_data;

        self.veh_data_stats.generated_data_size += v2v_payload.total_data_size;

        let v2v_links_opt = core_state.vanet.mesh_links.v2v_link_cache.remove(&self.id);
        let v2v_links = match self.linker {
            VehLinkerType::Simple(ref linker) => linker.find_vehicle_mesh_links(v2v_links_opt),
        };

        self.veh_data_stats.outgoing_v2v_data_size = 0.0;
        match v2v_links {
            Some(v2v_links) => {
                for neighbour_device_id in v2v_links {
                    core_state
                        .vanet
                        .payloads
                        .v2v_data
                        .entry(neighbour_device_id)
                        .and_modify(|payload| payload.push(v2v_payload.clone()))
                        .or_insert(vec![v2v_payload.clone()]);
                    self.veh_data_stats.outgoing_v2v_data_size += v2v_payload.total_data_size;
                }
            }
            None => {}
        }
    }

    fn transfer_data_to_bs(&mut self, core_state: &mut Core) {
        let mut v2bs_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::BaseStation),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        v2bs_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2bs_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2bs_payload),
        };
        v2bs_payload.id = self.id;
        v2bs_payload.sensor_data = self.sensor_data;

        self.veh_data_stats.generated_data_size += v2bs_payload.total_data_size;

        let v2bs_links_opt = core_state
            .vanet
            .infra_links
            .v2bs_link_cache
            .remove(&self.id);
        let selected_bs_id = match self.linker {
            VehLinkerType::Simple(ref linker) => linker.find_bs_link(v2bs_links_opt),
        };
        self.veh_data_stats.assigned_bs_id = selected_bs_id;

        self.veh_data_stats.outgoing_v2bs_data_size = 0.0;
        match selected_bs_id {
            Some(bs_id) => {
                core_state
                    .vanet
                    .payloads
                    .v2bs_data
                    .entry(bs_id)
                    .and_modify(|payload| payload.push(v2bs_payload.clone()))
                    .or_insert(vec![v2bs_payload.clone()]);
                self.veh_data_stats.outgoing_v2bs_data_size += v2bs_payload.total_data_size;
            }
            None => {}
        }
    }

    fn transfer_data_to_rsu(&mut self, core_state: &mut Core) {
        let mut v2rsu_payload = match self.composer {
            ComposerType::Basic(ref composer) => composer.compose_payload(DeviceType::RSU),
            ComposerType::Random(ref composer) => composer.compose_payload(),
        };
        v2rsu_payload = match self.simplifier {
            SimplifierType::Basic(ref simplifier) => simplifier.simplify_payload(v2rsu_payload),
            SimplifierType::Random(ref simplifier) => simplifier.simplify_payload(v2rsu_payload),
        };
        v2rsu_payload.id = self.id;
        v2rsu_payload.sensor_data = self.sensor_data;

        self.veh_data_stats.generated_data_size += v2rsu_payload.total_data_size;

        let v2rsu_links_opt = core_state
            .vanet
            .mesh_links
            .v2rsu_link_cache
            .remove(&self.id);

        let selected_rsu_id = match self.linker {
            VehLinkerType::Simple(ref linker) => linker.find_rsu_link(v2rsu_links_opt),
        };
        self.veh_data_stats.assigned_rsu_id = selected_rsu_id;

        self.veh_data_stats.outgoing_v2rsu_data_size = 0.0;
        match selected_rsu_id {
            Some(rsu_id) => {
                core_state
                    .vanet
                    .mesh_links
                    .rsu2v_link_cache
                    .entry(rsu_id)
                    .and_modify(|links| {
                        links.push(self.id);
                    })
                    .or_insert(vec![self.id]);
                core_state
                    .vanet
                    .payloads
                    .v2rsu_data
                    .entry(rsu_id)
                    .and_modify(|payload| {
                        payload.push(v2rsu_payload.clone());
                    })
                    .or_insert(vec![v2rsu_payload.clone()]);
                self.veh_data_stats.outgoing_v2rsu_data_size += v2rsu_payload.total_data_size;
            }
            None => {}
        }
    }

    fn collect_data(&mut self, core_state: &mut Core) {}

    pub fn as_agent(self) -> Box<dyn Agent> {
        Box::new(self)
    }
}

impl Agent for Vehicle {
    fn step(&mut self, state: &mut dyn State) {
        debug!("{} is active", self.id);
        self.status = DeviceState::Active;
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.step = core_state.step;

        self.update_geo_data(core_state);
        self.veh_data_stats.generated_data_size = 0.0;
        self.transfer_data_to_bs(core_state);
        self.transfer_data_to_rsu(core_state);
        self.transfer_data_to_vehicles(core_state);
        self.storage += self.veh_data_stats.generated_data_size;

        // Initiate deactivation if it is time
        if self.step == self.timing.peek_deactivation_time() {
            self.deactivate(core_state);
        }
    }

    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        self.status == DeviceState::Inactive
    }

    fn before_step(
        &mut self,
        state: &mut dyn State,
    ) -> Option<Vec<(Box<dyn Agent>, ScheduleOptions)>> {
        debug!("Before step {}", self.step);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.collect_data(core_state);
        None
    }

    fn after_step(
        &mut self,
        _state: &mut dyn State,
    ) -> Option<Vec<(Box<dyn Agent>, ScheduleOptions)>> {
        None
    }
}

impl Hash for Vehicle {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for Vehicle {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, loc: Real2D) {
        self.location = loc;
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for Vehicle {}

impl PartialEq for Vehicle {
    fn eq(&self, other: &Vehicle) -> bool {
        self.id == other.id
    }
}
