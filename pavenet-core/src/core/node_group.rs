use krabmaga::engine::schedule::Schedule;
use pavenet_config::types::ts::TimeStamp;

pub trait NodeGroup {
    fn init(&mut self, schedule: &mut Schedule);
    fn before_step(&mut self, step: TimeStamp);
    fn update(&mut self, step: TimeStamp);
    fn after_step(&mut self, schedule: &mut Schedule);
    fn streaming_step(&mut self, step: TimeStamp);
}
