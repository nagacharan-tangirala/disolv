use disolv_core::agent::{AgentClass, AgentId, AgentProperties};
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType};
use hashbrown::HashMap;

use crate::net::radio::CommStats;

#[derive(Clone, Debug, Default)]
pub struct FlowRegister {
    pub comm_stats: CommStats,
    pub out_link_agents: HashMap<AgentClass, Vec<AgentId>>,
    pub in_link_agents: HashMap<AgentClass, Vec<AgentId>>,
}

impl FlowRegister {
    pub fn reset(&mut self) {
        self.comm_stats.reset();
        self.in_link_agents.clear();
        self.out_link_agents.clear();
    }

    pub fn register_outgoing_attempt<
        C: ContentType,
        D: DataUnit<C>,
        M: Metadata,
        P: AgentProperties,
        Q: QueryType,
    >(
        &mut self,
        payload: &Payload<C, D, M, P, Q>,
    ) {
        self.comm_stats
            .outgoing_stats
            .add_attempted(&payload.metadata);
    }

    pub fn register_outgoing_feasible<
        C: ContentType,
        D: DataUnit<C>,
        M: Metadata,
        P: AgentProperties,
        Q: QueryType,
    >(
        &mut self,
        payload: &Payload<C, D, M, P, Q>,
    ) {
        self.comm_stats
            .outgoing_stats
            .add_feasible(&payload.metadata);
        self.out_link_agents
            .entry(*payload.agent_state.class())
            .or_default()
            .push(payload.agent_state.id());
    }

    pub fn register_incoming<
        C: ContentType,
        D: DataUnit<C>,
        M: Metadata,
        P: AgentProperties,
        Q: QueryType,
    >(
        &mut self,
        payloads: &[Payload<C, D, M, P, Q>],
    ) {
        payloads.iter().for_each(|payload| {
            self.comm_stats.incoming_stats.update(&payload.metadata);
            self.in_link_agents
                .entry(*payload.agent_state.class())
                .or_default()
                .push(payload.agent_state.id());
        });
    }
}
