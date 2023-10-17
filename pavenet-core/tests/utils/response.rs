use pavenet_core::response::{Queryable, RequestCreek, TransferStats};

#[derive(Clone, Copy)]
pub(crate) enum DataType {
    Status,
    Image,
}

impl Queryable for DataType {}

#[derive(Clone, Copy)]
pub(crate) struct RequestInfo {
    pub(crate) request_type: DataType,
    pub(crate) size: f32,
}

impl RequestCreek<DataType> for RequestInfo {}

#[derive(Clone, Copy)]
pub(crate) struct TransferInfo {
    pub(crate) latency: f32,
}

impl TransferStats for TransferInfo {}
