use crate::core::core::Core;

pub trait Payload {
    fn collect_from_sensors(&mut self);
    fn build_payload(&mut self) -> Box<dyn Payload>;
}

pub trait Transmitter {
    fn generate_data(&mut self, core: &mut Core);
    fn transmit(&mut self, payload: Box<dyn Payload>);
}
