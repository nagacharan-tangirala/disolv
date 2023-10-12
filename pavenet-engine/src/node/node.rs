use crate::engine::engine::Engine;
use crate::node::power::PowerState;
use downcast_rs::Downcast;
use dyn_clone::DynClone;

pub trait Node: Send + Sync + Downcast + DynClone {
    fn power_state(&self) -> PowerState;
    fn node_order(&self) -> i32;
    fn set_power_state(&mut self, power_state: PowerState);
    fn step(&mut self, engine: &mut Engine);
    fn after_step(&mut self, engine: &mut Engine);
}
