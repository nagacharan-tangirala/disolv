use pavenet_core::structs::MapState;
use pavenet_engine::engine::engine::Engine;

pub trait Mapper {
    fn map_state(&self) -> MapState;
    fn set_map_state(&mut self, map_state: MapState);
}

pub trait Transmitter {
    type Item;
    fn collect_downstream(&mut self) -> Vec<Self::Item>;
    fn generate_data(&mut self, engine: &mut Engine) -> Self::Item;
    fn transmit(&mut self, data: Self::Item);
}

pub trait Recipient {
    type Item;
    fn receive(&mut self, data: &Vec<Self::Item>);
    fn report_stats(&mut self, engine: &mut Engine);
}
