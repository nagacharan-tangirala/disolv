use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::fl::server::ServerState;
use crate::models::ai::models::ClientState;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct ServerDurations {
    pub(crate) initiation: TimeMS,
    pub(crate) analysis: TimeMS,
    pub(crate) selection: TimeMS,
    pub(crate) broadcast: TimeMS,
    pub(crate) aggregation: TimeMS,
    pub(crate) round_length: TimeMS,
}

#[derive(Debug, Clone, Copy, TypedBuilder)]
pub(crate) struct ServerTimes {
    pub(crate) durations: ServerDurations,
    pub(crate) next_change_at: TimeMS,
}

impl ServerTimes {
    pub(crate) fn new(durations: ServerDurations) -> Self {
        Self {
            durations,
            next_change_at: TimeMS::default(),
        }
    }

    pub(crate) fn update_time(&mut self, now: TimeMS, current_state: ServerState) {
        self.next_change_at = now
            + match current_state {
                ServerState::Idle => self.durations.initiation,
                ServerState::ClientAnalysis => self.durations.analysis,
                ServerState::Aggregation => self.durations.aggregation,
                ServerState::ClientSelection => self.durations.selection,
                ServerState::TrainingRound => self.durations.round_length,
                ServerState::GlobalUpdate => self.durations.broadcast,
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

#[derive(Debug, Clone, Copy, TypedBuilder)]
pub(crate) struct ClientTimes {
    pub(crate) durations: ClientDurations,
    pub(crate) next_change_at: TimeMS,
}

impl ClientTimes {
    pub(crate) fn new(durations: ClientDurations) -> Self {
        Self {
            durations,
            next_change_at: TimeMS::default(),
        }
    }

    // Add training duration when switching from sensing. Switching away from sensing should
    // happen only when sufficient time is spent in sensing so that data is collected.
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
