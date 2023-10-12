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
