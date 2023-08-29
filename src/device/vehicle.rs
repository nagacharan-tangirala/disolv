use core::fmt;
use std::hash::{Hash, Hasher};

use crate::models::mobility::{Mobility, TraceMobility};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::rand;
use krabmaga::rand::Rng;

use crate::sim::network::Network;

/// The most basic agent should implement Clone, Copy and Agent to be able to be inserted in a Schedule.
#[derive(Clone, Copy)]
pub struct Vehicle {
    pub id: u32,
    pub loc: Real2D,
    timing: Timing,
    mobility: TraceMobility,
}

#[derive(Clone, Copy)]
struct Timing {
    activation: u64,
    deactivation: u64,
}

impl Agent for Vehicle {
    /// Put the code that should happen for each step, for each agent here.
    fn step(&mut self, state: &mut dyn State) {
        let state = state.as_any().downcast_ref::<Network>().unwrap();
        let mut rng = rand::thread_rng();

        self.loc = Real2D { x: 0.0, y: 0.0 };

        state
            .vehicle_field
            .set_object_location(*self, Real2D { x: 0.0, y: 0.0 });
    }

    /// Put the code that decides if an agent should be removed or not
    /// for example in simulation where agents can die
    fn is_stopped(&mut self, _state: &mut dyn State) -> bool {
        false
    }
}

impl Hash for Vehicle {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Location2D<Real2D> for Vehicle {
    fn get_location(self) -> Real2D {
        self.loc
    }

    fn set_location(&mut self, loc: Real2D) {
        self.loc = loc;
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for Vehicle {}

impl PartialEq for Vehicle {
    fn eq(&self, other: &Vehicle) -> bool {
        self.id == other.id
    }
}
