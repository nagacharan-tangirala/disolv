use crate::node::power::PowerState;
use pavenet_core::enums::NodeType;

pub trait Node: Clone + Send + Sync + Default + 'static {
    fn node_type(&self) -> NodeType;
    fn power_state(&self) -> PowerState;
    fn node_order(&self) -> i32;
    fn set_power_state(&mut self, power_state: PowerState);
    fn step<U>(&mut self, pool: &mut U);
    fn after_step<U>(&mut self, pool: &mut U);
}
