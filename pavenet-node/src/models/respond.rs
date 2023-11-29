use pavenet_core::response::{DResponse, TransferMetrics};
use pavenet_engine::bucket::TimeS;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

#[derive(Clone, Debug, Copy)]
pub enum Responder {
    Stats(StatsResponder),
}

impl Responder {
    pub fn new(responder_settings: ResponderSettings) -> Self {
        match responder_settings.name.as_str() {
            "basic" => Responder::Stats(StatsResponder::default()),
            _ => panic!("Unknown responder type"),
        }
    }

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
    step: TimeS,
}

impl StatsResponder {
    pub(crate) fn new(_responder_settings: ResponderSettings) -> Self {
        Self {
            step: TimeS::default(),
        }
    }

    pub(crate) fn compose_response(
        &mut self,
        in_response: Option<DResponse>,
        transfer_stats: TransferMetrics,
    ) -> DResponse {
        let content = match in_response {
            Some(response) => response.content,
            None => None,
        };
        DResponse::new(transfer_stats, content)
    }
}
