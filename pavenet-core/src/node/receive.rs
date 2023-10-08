use crate::core::core::Core;
use crate::node::transmit::Payload;

pub trait Recipient {
    fn receive(&mut self, payloads: &mut Vec<Box<dyn Payload>>);
    fn report_stats(&mut self, core: &mut Core);
}
