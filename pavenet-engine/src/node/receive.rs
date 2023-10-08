use crate::engine::core::Core;
use crate::node::transmit::Transferable;

pub trait Recipient {
    fn receive(&mut self, payloads: &mut Vec<Box<dyn Transferable>>);
    fn report_stats(&mut self, core: &mut Core);
}
