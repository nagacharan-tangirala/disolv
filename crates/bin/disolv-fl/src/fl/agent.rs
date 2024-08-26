use burn::tensor::backend::AutodiffBackend;

use disolv_core::agent::{Activatable, Agent, AgentId, AgentOrder, Movable, Orderable};
use disolv_core::bucket::TimeMS;
use disolv_models::device::mobility::MapState;

use crate::fl::bucket::FlBucket;
use crate::fl::client::Client;
use crate::fl::server::Server;

#[derive(Clone)]
pub enum FAgent<A: AutodiffBackend> {
    Client(Client<A>),
    Server(Server<A>),
}

impl<A: AutodiffBackend> Orderable for FAgent<A> {
    fn order(&self) -> AgentOrder {
        match self {
            FAgent::Client(client) => client.order(),
            FAgent::Server(server) => server.order(),
        }
    }
}

impl<A: AutodiffBackend> Activatable<FlBucket<A>> for FAgent<A> {
    fn activate(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.activate(bucket),
            FAgent::Server(server) => server.activate(bucket),
        }
    }

    fn deactivate(&mut self) {
        match self {
            FAgent::Client(client) => client.deactivate(),
            FAgent::Server(server) => server.deactivate(),
        }
    }

    fn is_deactivated(&self) -> bool {
        match self {
            FAgent::Client(client) => client.is_deactivated(),
            FAgent::Server(server) => server.is_deactivated(),
        }
    }

    fn has_activation(&self) -> bool {
        match self {
            FAgent::Client(client) => client.has_activation(),
            FAgent::Server(server) => server.has_activation(),
        }
    }

    fn time_of_activation(&mut self) -> TimeMS {
        match self {
            FAgent::Client(client) => client.time_of_activation(),
            FAgent::Server(server) => server.time_of_activation(),
        }
    }
}

impl<A: AutodiffBackend> Movable<FlBucket<A>> for FAgent<A> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        match self {
            FAgent::Client(client) => client.mobility(),
            FAgent::Server(server) => server.mobility(),
        }
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.set_mobility(bucket),
            FAgent::Server(server) => server.set_mobility(bucket),
        }
    }
}

impl<A: AutodiffBackend> Agent<FlBucket<A>> for FAgent<A> {
    fn id(&self) -> AgentId {
        match self {
            FAgent::Client(client) => client.id(),
            FAgent::Server(server) => server.id(),
        }
    }

    fn stage_one(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.stage_one(bucket),
            FAgent::Server(server) => server.stage_one(bucket),
        }
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.stage_two_reverse(bucket),
            FAgent::Server(server) => server.stage_two_reverse(bucket),
        }
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.stage_three(bucket),
            FAgent::Server(server) => server.stage_three(bucket),
        }
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.stage_four_reverse(bucket),
            FAgent::Server(server) => server.stage_four_reverse(bucket),
        }
    }

    fn stage_five(&mut self, bucket: &mut FlBucket<A>) {
        match self {
            FAgent::Client(client) => client.stage_five(bucket),
            FAgent::Server(server) => server.stage_five(bucket),
        }
    }
}
