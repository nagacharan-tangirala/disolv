use disolv_core::agent::{AgentClass, AgentProperties};
use disolv_core::bucket::TimeMS;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType};
use disolv_core::radio::{Link, LinkFeatures};

use crate::net::radio::CommStats;

/// Allows a model to support building a payload that is used for transmission.
pub trait Compose<C, D, M, P, Q>: Clone
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn compose(
        &self,
        at: TimeMS,
        target_class: &AgentClass,
        agent_state: &P,
    ) -> Payload<C, D, M, P, Q>;
}

/// Add this to a struct/enum that implements link selection based on any communication statistics.
pub trait LinkSelect<F>: Clone
where
    F: LinkFeatures,
{
    fn select_link(&self, links: Vec<Link<F>>, agent_stats: &Vec<&CommStats>) -> Vec<Link<F>>;
}
