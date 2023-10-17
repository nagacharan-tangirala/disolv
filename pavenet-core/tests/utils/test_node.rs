use crate::utils::bucket::MyBucket;
use crate::utils::payload::{PayloadStatData, SensorData};
use crate::utils::response::DataType;
use crate::utils::types::{Nid, Order, Ts};
use pavenet_core::mobility::{MobilityInfo, Movable};
use pavenet_core::payload::PayloadData;
use pavenet_core::tier::Tiered;
use pavenet_core::uplink::{DataMaker, Gatherer};
use pavenet_engine::entity::{Entity, Kind};
use pavenet_engine::node::Node;
use std::fmt::Display;

pub(crate) type MyNode = Node<Nid, TDevice, DeviceType, MyBucket, Ts>;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DeviceType {
    #[default]
    TypeA,
    TypeB,
}

impl Kind for DeviceType {}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::TypeA => write!(f, "TypeA"),
            DeviceType::TypeB => write!(f, "TypeB"),
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub(crate) struct TDevice {
    pub(crate) id: Nid,
    pub(crate) device_type: DeviceType,
    pub(crate) order: Order,
    pub(crate) step: Ts,
    pub(crate) mobility: Mobility,
}

impl Entity<MyBucket, Ts> for TDevice {
    fn step(&mut self, bucket: &mut MyBucket) {
        self.step = bucket.step;
        println!("step {} in TDevice of type {}", self.step, self.device_type);
    }
    fn after_step(&mut self, _bucket: &mut MyBucket) {
        println!("after_step in TDevice of type {}", self.device_type);
    }
    fn is_stopped(&self) -> bool {
        false
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Mobility {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) velocity: f32,
}

impl Mobility {
    fn new(x: f32, y: f32, velocity: f32) -> Mobility {
        Mobility { x, y, velocity }
    }
}

impl MobilityInfo for Mobility {}

impl Movable<Mobility> for TDevice {
    fn mobility(&self) -> &Mobility {
        &self.mobility
    }

    fn set_mobility(&mut self, mobility: Mobility) {
        self.mobility = mobility;
    }
}

pub(crate) fn make_device(id: Nid, device_type: DeviceType, order: Order) -> TDevice {
    TDevice {
        id,
        device_type,
        order,
        mobility: Mobility::default(),
        step: Ts::default(),
    }
}

pub(crate) fn as_node(device: TDevice) -> MyNode {
    Node::new(device.id, device, device.device_type)
}

impl Tiered<Order> for TDevice {
    fn tier(&self) -> &Order {
        &self.order
    }
    fn set_tier(&mut self, tier: Order) {
        self.order = tier;
    }
}

pub(crate) type MyPayloadData = PayloadData<Nid, SensorData, DataType>;

impl Gatherer<Ts, Nid, SensorData, MyBucket, DataType> for TDevice {
    fn gather(&mut self, _bucket: &mut MyBucket) -> Option<Vec<MyPayloadData>> {
        let mut data = Vec::new();
        let mut data_pile = SensorData {
            data_type: DataType::Status,
            size: 0.1,
        };
        for _ in 0..10 {
            data_pile.size += 1.0;
            data.push(PayloadData::new(data_pile, self.id));
        }
        Some(data)
    }
}

impl DataMaker<Nid, SensorData, DataType, PayloadStatData> for TDevice {
    fn make_data(&mut self) -> MyPayloadData {
        let data_pile = SensorData {
            data_type: DataType::Status,
            size: 0.1,
        };
        PayloadData::new(data_pile, self.id)
    }

    fn payload_stats(&mut self) -> PayloadStatData {
        PayloadStatData {
            data_size: 0.1,
            data_count: 1,
        }
    }
}
