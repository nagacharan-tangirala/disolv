use crate::model::{Model, ModelSettings};
use pavenet_core::message::{DResponse, TransferMetrics};
use pavenet_engine::bucket::TimeS;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

impl ModelSettings for ResponderSettings {}

#[derive(Clone, Debug, Copy)]
pub enum Responder {
    Stats(StatsResponder),
}

impl Model for Responder {
    type Settings = ResponderSettings;

    fn with_settings(settings: &ResponderSettings) -> Self {
        match settings.name.as_str() {
            "basic" => Responder::Stats(StatsResponder::default()),
            _ => panic!("Unknown responder type"),
        }
    }
}

impl Responder {
    pub fn compose_response(
        &mut self,
        in_response: Option<DResponse>,
        transfer_stats: TransferMetrics,
    ) -> DResponse {
        match self {
            Responder::Stats(responder) => responder.compose_response(in_response, transfer_stats),
        }
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub struct StatsResponder {
    _step: TimeS,
}

impl StatsResponder {
    pub fn new(_responder_settings: ResponderSettings) -> Self {
        Self {
            _step: TimeS::default(),
        }
    }

    pub fn compose_response(
        &mut self,
        in_response: Option<DResponse>,
        transfer_stats: TransferMetrics,
    ) -> DResponse {
        let downstream = match in_response {
            Some(response) => response.downstream,
            None => None,
        };
        DResponse::builder()
            .reply(None)
            .tx_status(transfer_stats)
            .downstream(downstream)
            .build()
    }
}
