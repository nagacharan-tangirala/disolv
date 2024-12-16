use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;
use disolv_core::model::{Model, ModelSettings};

use crate::fl::server::ServerState;
use crate::models::ai::models::ClientState;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct ServerDurations {
    pub(crate) initiation: TimeMS,
    pub(crate) analysis: TimeMS,
    pub(crate) selection: TimeMS,
    pub(crate) aggregation: TimeMS,
    pub(crate) round_length: TimeMS,
}

impl ModelSettings for ServerDurations {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ServerTimes {
    pub(crate) durations: ServerDurations,
    pub(crate) next_change_at: TimeMS,
}

impl Model for ServerTimes {
    type Settings = ServerDurations;

    fn with_settings(settings: &Self::Settings) -> Self {
        Self {
            durations: settings.clone(),
            next_change_at: TimeMS::default(),
        }
    }
}

impl ServerTimes {
    pub(crate) fn update_time(&mut self, now: TimeMS, current_state: ServerState) {
        self.next_change_at = now
            + match current_state {
                ServerState::Idle => self.durations.initiation,
                ServerState::ClientAnalysis => self.durations.analysis,
                ServerState::ClientSelection => self.durations.selection,
                ServerState::TrainingRound => self.durations.round_length,
                ServerState::Aggregation => self.durations.aggregation,
            }
    }

    pub(crate) fn is_time_to_change(&self, now: TimeMS) -> bool {
        now == self.next_change_at
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct ClientDurations {
    pub(crate) training: TimeMS,
    pub(crate) sensing: TimeMS,
}

impl ModelSettings for ClientDurations {}

#[derive(Debug, Clone, Copy, TypedBuilder)]
pub(crate) struct ClientTimes {
    pub(crate) durations: ClientDurations,
    pub(crate) next_change_at: TimeMS,
}

impl Model for ClientTimes {
    type Settings = ClientDurations;

    fn with_settings(settings: &Self::Settings) -> Self {
        Self {
            durations: settings.clone(),
            next_change_at: TimeMS::default(),
        }
    }
}

impl ClientTimes {
    pub(crate) fn update_time(&mut self, now: TimeMS, current_state: ClientState) {
        self.next_change_at = now
            + match current_state {
                ClientState::Sensing => self.durations.training,
                _ => self.durations.sensing,
            }
    }

    pub(crate) fn is_time_to_change(&self, now: TimeMS) -> bool {
        now == self.next_change_at
    }
}
