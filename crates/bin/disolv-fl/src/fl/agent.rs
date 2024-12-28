use burn::tensor::backend::AutodiffBackend;

use disolv_core::bucket::TimeMS;

use crate::fl::bucket::FlBucket;
use crate::fl::client::Client;
use crate::fl::server::Server;
use crate::models::ai::compose::FlMessageDraft;
use crate::models::device::message::FlPayload;

#[derive(Clone)]
pub enum FAgent<B: AutodiffBackend> {
    FClient(Client<B>),
    FServer(Server<B>),
}

impl<B: AutodiffBackend> FAgent<B> {
    pub(crate) fn init(&mut self, bucket: &mut FlBucket<B>) {
        match self {
            FAgent::FClient(client) => client.init(bucket),
            FAgent::FServer(server) => server.init(bucket),
        }
    }

    pub(crate) fn agent_state(&self) -> String {
        match self {
            FAgent::FClient(client) => client.client_state.to_string(),
            FAgent::FServer(server) => server.server_state.to_string(),
        }
    }

    pub(crate) fn update_step(&mut self, new_step: TimeMS) {
        match self {
            FAgent::FClient(client) => client.step = new_step,
            FAgent::FServer(server) => server.step = new_step,
        }
    }

    pub(crate) fn draft_fl_message(&mut self, bucket: &mut FlBucket<B>) -> FlMessageDraft {
        match self {
            FAgent::FClient(client) => client.draft_fl_message(bucket),
            FAgent::FServer(server) => server.draft_fl_message(bucket),
        }
    }

    pub(crate) fn handle_incoming(&mut self, bucket: &mut FlBucket<B>, payloads: &mut [FlPayload]) {
        match self {
            FAgent::FClient(client) => client.handle_incoming(bucket, payloads),
            FAgent::FServer(server) => server.handle_incoming(bucket, payloads),
        }
    }
}
