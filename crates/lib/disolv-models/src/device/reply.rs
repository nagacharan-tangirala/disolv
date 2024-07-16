use serde::Deserialize;

use disolv_core::bucket::TimeMS;
use disolv_core::model::{Model, ModelSettings};

use crate::net::message::{TxMetrics, V2XResponse};

#[derive(Deserialize, Debug, Clone)]
pub struct ReplierSettings {
    pub name: String,
}

impl ModelSettings for ReplierSettings {}

#[derive(Clone, Debug, Copy)]
pub enum Replier {
    Stats(StatsReplier),
}

impl Model for Replier {
    type Settings = ReplierSettings;

    fn with_settings(settings: &ReplierSettings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "stats" => Replier::Stats(StatsReplier::default()),
            _ => panic!("Unsupported replier type {}.", settings.name),
        }
    }
}

impl Replier {
    pub fn compose_response(
        &mut self,
        in_response: Option<V2XResponse>,
        transfer_stats: TxMetrics,
    ) -> V2XResponse {
        match self {
            Replier::Stats(responder) => responder.compose_response(in_response, transfer_stats),
        }
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub struct StatsReplier {
    _step: TimeMS,
}

impl StatsReplier {
    pub fn new(_replier_settings: ReplierSettings) -> Self {
        Self {
            _step: TimeMS::default(),
        }
    }

    pub fn compose_response(
        &mut self,
        in_response: Option<V2XResponse>,
        transfer_stats: TxMetrics,
    ) -> V2XResponse {
        let downstream = match in_response {
            Some(response) => response.downstream,
            None => None,
        };
        V2XResponse::builder()
            .reply(None)
            .tx_report(transfer_stats)
            .downstream(downstream)
            .build()
    }
}
