use crate::core::core::Core;
use crate::node::power::PowerState;
use crate::node::receive::Recipient;
use crate::node::transmit::Transmitter;
use downcast_rs::Downcast;
use dyn_clone::DynClone;

pub trait Node: Transmitter + Recipient + Send + Sync + Downcast + DynClone {
    fn power_state(&self) -> PowerState;
    fn node_order(&self) -> i32;
    fn set_power_state(&mut self, power_state: PowerState);
    fn step(&mut self, core: &mut Core);
    fn after_step(&mut self, core: &mut Core);
}
