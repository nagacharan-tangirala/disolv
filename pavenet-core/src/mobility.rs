pub trait MobilityInfo: Copy + Clone {}

pub trait Movable<T>
where
    T: MobilityInfo,
{
    fn mobility(&self) -> &T;
    fn set_mobility(&mut self, mobility_info: T);
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::mobility::{MobilityInfo, Movable};

    #[derive(Copy, Clone, Default, Debug)]
    struct Mobility {
        x: f32,
        y: f32,
        velocity: f32,
    }

    impl Mobility {
        fn new(x: f32, y: f32, velocity: f32) -> Mobility {
            Mobility { x, y, velocity }
        }
    }

    impl MobilityInfo for Mobility {}

    #[derive(Copy, Clone, Default, Debug)]
    struct Device {
        mobility: Mobility,
    }

    impl Movable<Mobility> for Device {
        fn mobility(&self) -> &Mobility {
            &self.mobility
        }

        fn set_mobility(&mut self, mobility_info: Mobility) {
            self.mobility = mobility_info;
        }
    }

    #[test]
    fn test_mobility() {
        let mut device = Device::default();
        let mobility = Mobility::new(1.0, 2.0, 3.0);
        device.set_mobility(mobility);
        assert_eq!(device.mobility().x, 1.0);
        assert_eq!(device.mobility().y, 2.0);
        assert_eq!(device.mobility().velocity, 3.0);
    }
}
