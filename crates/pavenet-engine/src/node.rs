use crate::bucket::Bucket;
use crate::engine::GEngine;
use crate::entity::{Entity, Tier};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use serde::Deserialize;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Deserialize, Default, Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for NodeId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>()?;
        Ok(Self(id))
    }
}

impl From<u32> for NodeId {
    fn from(f: u32) -> Self {
        Self(f)
    }
}

impl From<i64> for NodeId {
    fn from(f: i64) -> Self {
        Self(f as u32)
    }
}

impl From<i32> for NodeId {
    fn from(f: i32) -> Self {
        Self(f as u32)
    }
}

impl NodeId {
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    pub node_id: NodeId,
    pub entity: E,
    _marker: std::marker::PhantomData<fn() -> (B, T)>,
}

impl<B, E, T> Agent for GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    fn before_step(&mut self, state: &mut dyn State) {
        let engine: &mut GEngine<B> = state.as_any_mut().downcast_mut::<GEngine<B>>().unwrap();
        self.entity.uplink_stage(&mut engine.bucket);
    }

    fn step(&mut self, state: &mut dyn State) {
        let engine = state.as_any_mut().downcast_mut::<GEngine<B>>().unwrap();
        self.entity.sidelink_stage(&mut engine.bucket);
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state.as_any_mut().downcast_mut::<GEngine<B>>().unwrap();
        self.entity.downlink_stage(&mut engine.bucket);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.entity.is_stopped()
    }
}

impl<B, E, T> GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    pub fn new(node_id: NodeId, node: E) -> Self {
        Self {
            node_id,
            entity: node,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B, E, T> Hash for GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<B, E, T> fmt::Display for GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<B, E, T> PartialEq for GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    fn eq(&self, other: &GNode<B, E, T>) -> bool {
        self.node_id == other.node_id
    }
}

impl<B, E, T> Eq for GNode<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
}
