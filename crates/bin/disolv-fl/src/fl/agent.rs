use burn::tensor::backend::AutodiffBackend;

use disolv_core::agent::{Activatable, Agent, AgentId, AgentOrder, Movable, Orderable};
use disolv_core::bucket::TimeMS;
use disolv_models::device::mobility::MapState;

use crate::fl::bucket::FlBucket;
use crate::fl::client::Client;
use crate::fl::server::Server;

#[derive(Clone)]
pub enum FAgent<A: AutodiffBackend> {
    FClient(Client<A>),
    FServer(Server<A>),
}

impl<A: AutodiffBackend> Orderable for FAgent<A> {
    fn order(&self) -> AgentOrder {
        match self {
            FAgent::FClient(client) => client.order(),
            FAgent::FServer(server) => server.order(),
        }
    }
}

impl<A: AutodiffBackend> Activatable<FlBucket<A>> for FAgent<A> {
    fn activate(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.activate(bucket),
            FAgent::FServer(server) => server.activate(bucket),
        }
    }

    fn deactivate(&mut self) {
        match self {
            FAgent::FClient(client) => client.deactivate(),
            FAgent::FServer(server) => server.deactivate(),
        }
    }

    fn is_deactivated(&self) -> bool {
        match self {
            FAgent::FClient(client) => client.is_deactivated(),
            FAgent::FServer(server) => server.is_deactivated(),
        }
    }

    fn has_activation(&self) -> bool {
        match self {
            FAgent::FClient(client) => client.has_activation(),
            FAgent::FServer(server) => server.has_activation(),
        }
    }

    fn time_of_activation(&mut self) -> TimeMS {
        match self {
            FAgent::FClient(client) => client.time_of_activation(),
            FAgent::FServer(server) => server.time_of_activation(),
        }
    }
}

impl<A: AutodiffBackend> Movable<FlBucket<A>> for FAgent<A> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        match self {
            FAgent::FClient(client) => client.mobility(),
            FAgent::FServer(server) => server.mobility(),
        }
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.set_mobility(bucket),
            FAgent::FServer(server) => server.set_mobility(bucket),
        }
    }
}

impl<A: AutodiffBackend> Agent<FlBucket<A>> for FAgent<A> {
    fn id(&self) -> AgentId {
        match self {
            FAgent::FClient(client) => client.id(),
            FAgent::FServer(server) => server.id(),
        }
    }

    fn stage_one(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.stage_one(bucket),
            FAgent::FServer(server) => server.stage_one(bucket),
        }
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.stage_two_reverse(bucket),
            FAgent::FServer(server) => server.stage_two_reverse(bucket),
        }
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.stage_three(bucket),
            FAgent::FServer(server) => server.stage_three(bucket),
        }
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.stage_four_reverse(bucket),
            FAgent::FServer(server) => server.stage_four_reverse(bucket),
        }
    }

    fn stage_five(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::FClient(client) => client.stage_five(bucket),
            FAgent::FServer(server) => server.stage_five(bucket),
        }
    }
}
