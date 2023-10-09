use crate::engine::core::Core;

pub trait Transferable {
    fn sensor_data(&mut self);
    fn collect_downstream(&mut self);
    fn build_payload(&mut self) -> Box<dyn Transferable>;
}

pub trait Transmitter {
    fn generate_data(&mut self, core: &mut Core) -> Box<dyn Transferable>;
    fn transmit(&mut self, payload: Box<dyn Transferable>);
}
