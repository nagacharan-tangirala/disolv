use pavenet_core::response::{Queryable, RequestCreek, TransferStats};

#[derive(Clone, Copy, Default)]
pub(crate) enum DataType {
    #[default]
    Status,
    Image,
}

impl Queryable for DataType {}

#[derive(Clone, Copy, Default)]
pub(crate) struct RequestInfo {
    pub(crate) request_type: DataType,
    pub(crate) size: f32,
}

impl RequestInfo {
    pub(crate) fn new(request_type: DataType, size: f32) -> Self {
        Self { request_type, size }
    }
}

impl RequestCreek<DataType> for RequestInfo {}

#[derive(Clone, Copy, Default)]
pub(crate) struct TransferInfo {
    pub(crate) latency: f32,
}

impl TransferStats for TransferInfo {}
