use crate::node::transmit::Transferable;

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

impl Transferable for TestPayload {
    fn sensor_data(&mut self, payload: &mut Box<dyn Transferable>) {
        println!("TestPayload::collect_from_sensors");
        self.sensor_data = 1.0;
    }

    fn collect_downstream(&mut self, payload: &mut Box<dyn Transferable>) {
        todo!()
    }

    fn build_payload(&mut self) -> Box<dyn Transferable> {
        Box::new(TestPayload::new(self.a, self.b))
    }
}
