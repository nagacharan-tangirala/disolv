use crate::node::transmit::Payload;

pub trait Recipient {
    fn receive(&self, payloads: &mut Vec<Box<dyn Payload>>);
}
