use crate::node::transmit::Payload;

pub trait Recipient {
    fn receive(&mut self, payloads: &mut Vec<Box<dyn Payload>>);
}
