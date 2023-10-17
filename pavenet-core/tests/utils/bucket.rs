use crate::utils::test_node::MyNode;
use crate::utils::types::{Nid, Ts};
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use pavenet_engine::bucket::Bucket;

#[derive(Default, Clone)]
pub struct MyBucket {
    pub step: Ts,
    pub devices: HashMap<Nid, MyNode>,
}

impl MyBucket {
    pub(crate) fn add(&mut self, node: MyNode) {
        self.devices.insert(node.node.id, node);
    }

    pub(crate) fn add_to_schedule(&mut self, schedule: &mut Schedule) {
        for (_, node) in self.devices.iter_mut() {
            schedule.schedule_repeating(
                Box::new(node.clone()),
                node.node_id.into(),
                0.,
                node.node.order.into(),
            );
        }
    }
}

impl Bucket<Ts> for MyBucket {
    fn init(&mut self, schedule: &mut Schedule) {
        self.add_to_schedule(schedule);
        self.step = Ts::default();
    }

    fn before_step(&mut self, _schedule: &mut Schedule) {
        println!("Before step in MyBucket");
    }

    fn update(&mut self, step: Ts) {
        self.step = step;
        println!("Update in MyBucket at {}", step);
    }

    fn after_step(&mut self, _schedule: &mut Schedule) {
        println!("After step in MyBucket");
    }

    fn streaming_step(&mut self, step: Ts) {
        println!("Streaming step in bucket at {}", step);
    }
}
