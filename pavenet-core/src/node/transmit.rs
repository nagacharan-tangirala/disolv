pub trait Payload {
    fn collect_from_sensors(&self);
    fn build_payload(&self) -> Box<dyn Payload>;
}

pub trait Transmitter {
    fn transmit(&self, payload: Box<dyn Payload>);
}
