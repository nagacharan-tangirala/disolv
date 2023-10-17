use crate::utils::bucket::MyBucket;
use crate::utils::payload::{PayloadStatData, SensorData};
use crate::utils::response::{DataType, RequestInfo, TransferInfo};
use crate::utils::types::{Nid, Order, Ts};
use pavenet_core::download::{Downloader, ResponseMaker};
use pavenet_core::mobility::{MobilityInfo, Movable};
use pavenet_core::payload::{Payload, PayloadData};
use pavenet_core::response::{RequestData, Response};
use pavenet_core::tier::Tiered;
use pavenet_core::upload::{DataMaker, Uploader};
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
    let device_type = device.device_type.clone();
    Node::new(device.id, device, device_type)
}

#[derive(Default, Clone, Debug)]
pub(crate) struct TDevice {
    pub(crate) id: Nid,
    pub(crate) device_type: DeviceType,
    pub(crate) order: Order,
    pub(crate) step: Ts,
    pub(crate) mobility: Mobility,
}

impl TDevice {
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

    fn transmit_data(&mut self, bucket: &mut MyBucket) {
        let mut payload = self.make_payload();
        let gather_data = self.gather(bucket);
        self.update_payload(&mut payload, gather_data);
        self.transmit(payload, bucket);
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

impl Entity<MyBucket, Ts> for TDevice {
    fn step(&mut self, bucket: &mut MyBucket) {
        self.step = bucket.step;
        self.transmit_data(bucket);
        println!("step {} in TDevice of type {}", self.step, self.device_type);
    }
    fn after_step(&mut self, _bucket: &mut MyBucket) {
        println!("after_step in TDevice of type {}", self.device_type);
    }
    fn is_stopped(&self) -> bool {
        false
    }
}

impl Tiered<Order> for TDevice {
    fn tier(&self) -> &Order {
        &self.order
    }
    fn set_tier(&mut self, tier: Order) {
        self.order = tier;
    }
}

pub(crate) type MyPayloadData = PayloadData<SensorData, Nid, DataType>;
pub type MyPayload = Payload<SensorData, Nid, PayloadStatData, DataType>;

impl DataMaker<SensorData, Nid, PayloadStatData, DataType> for TDevice {
    fn make_payload(&mut self) -> Payload<SensorData, Nid, PayloadStatData, DataType> {
        let payload_data = self.make_data();
        let payload_stats = self.payload_stats();
        let payload = Payload::new(payload_data, payload_stats);
        payload
    }

    fn update_payload(&mut self, given: &mut MyPayload, incoming: Option<Vec<MyPayload>>) {
        todo!()
    }
}

impl Uploader<MyBucket, SensorData, Nid, PayloadStatData, DataType, Ts> for TDevice {
    fn gather(&mut self, _bucket: &mut MyBucket) -> Option<Vec<MyPayload>> {
        let data_pile = SensorData {
            data_type: DataType::Status,
            size: 0.1,
        };
        let payload_data = PayloadData::new(data_pile, self.id);
        let payload_stats = PayloadStatData {
            data_size: 0.1,
            data_count: 10,
        };
        let payload = Payload::new(payload_data, payload_stats);
        Some(vec![payload])
    }

    fn transmit(&mut self, payload: MyPayload, bucket: &mut MyBucket) {
        bucket.add_payload(payload);
    }
}

pub(crate) type MyFeedbackData = RequestData<Nid, DataType, RequestInfo>;
pub(crate) type MyResponse = Response<Nid, DataType, RequestInfo, TransferInfo>;

impl ResponseMaker<MyBucket, Nid, DataType, RequestInfo, Ts, TransferInfo> for TDevice {
    fn read_response(&mut self, response: Response<Nid, DataType, RequestInfo, TransferInfo>) {
        todo!()
    }

    fn build_responses(
        &mut self,
        bucket: &mut MyBucket,
    ) -> Option<Vec<Response<Nid, DataType, RequestInfo, TransferInfo>>> {
        todo!()
    }
}

impl Downloader<DataType, Ts, TransferInfo, Nid, RequestInfo, MyBucket> for TDevice {
    fn fetch_feedback(
        &mut self,
        bucket: &mut MyBucket,
    ) -> Option<Response<Nid, DataType, RequestInfo, TransferInfo>> {
        todo!()
    }

    fn send_response(
        &mut self,
        responses: Option<Vec<Response<Nid, DataType, RequestInfo, TransferInfo>>>,
        bucket: MyBucket,
    ) {
        todo!()
    }
}
