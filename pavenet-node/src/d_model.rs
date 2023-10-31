use crate::models::compose::Composer;
use crate::models::power::PowerManager;
use crate::models::radio::Radio;
use crate::models::respond::Responder;
use crate::models::select::Selector;
use pavenet_core::bucket::TimeS;
use typed_builder::TypedBuilder;

pub trait BucketModel {
    fn init(&mut self, step: TimeS);
    fn stream_data(&mut self, step: TimeS);
    fn refresh_cache(&mut self, step: TimeS);
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub composer: Composer,
    pub responder: Responder,
    pub selector: Selector,
    pub radio: Radio,
    pub power: PowerManager,
}
