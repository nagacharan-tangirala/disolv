use pavenet_core::types::TimeStamp;
use serde::Deserialize;

pub trait TomlReadable: Deserialize<'static> + Clone {}

pub trait NodeModel: Copy + Clone {
    type Input: TomlReadable;
    fn to_input(&self) -> Self::Input;
}

pub trait PoolModel {
    fn init(&mut self, step: TimeStamp);
    fn stream_data(&mut self, step: TimeStamp);
    fn refresh_cache(&mut self, step: TimeStamp);
}
