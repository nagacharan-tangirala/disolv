use disolv_core::bucket::{Bucket, TimeMS};

#[derive(Default, Clone)]
pub struct MyBucket {
    pub step: TimeMS,
}

impl MyBucket {
    pub(crate) fn new() -> Self {
        Self {
            step: TimeMS::default(),
        }
    }
}

impl Bucket for MyBucket {
    fn initialize(&mut self, step: TimeMS) {
        self.step = step;
        println!("initialize in MyBucket");
    }

    fn before_agents(&mut self, step: TimeMS) {
        self.step = step;
        println!("before_agents in MyBucket");
    }

    fn after_stage_one(&mut self) {
        println!("after_stage_one in MyBucket");
    }

    fn after_stage_two(&mut self) {
        println!("after_stage_two in MyBucket");
    }

    fn after_stage_three(&mut self) {
        println!("after_stage_three in MyBucket");
    }

    fn after_stage_four(&mut self) {
        println!("after_stage_four in MyBucket");
    }

    fn after_agents(&mut self) {
        println!("after_agents in MyBucket");
    }

    fn stream_input(&mut self, step: TimeMS) {
        println!("Streaming step in bucket at {}", step);
    }

    fn stream_output(&mut self, step: TimeMS) {
        println!("Streaming step in bucket at {}", step);
    }

    fn terminate(self, step: TimeMS) {
        println!("End in MyBucket at {}", step);
    }
}
