use crate::node::transmit::Payload;

struct TestPayload {
    pub a: u32,
    pub b: u32,
    pub sensor_data: f32,
}

impl TestPayload {
    pub fn new(a: u32, b: u32) -> TestPayload {
        TestPayload {
            a,
            b,
            sensor_data: 0.0,
        }
    }
}

impl Payload for TestPayload {
    fn collect_from_sensors(&mut self) {
        println!("TestPayload::collect_from_sensors");
        self.sensor_data = 1.0;
    }

    fn build_payload(&mut self) -> Box<dyn Payload> {
        Box::new(TestPayload::new(self.a, self.b))
    }
}
