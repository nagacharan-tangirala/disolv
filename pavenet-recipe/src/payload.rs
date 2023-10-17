use crate::node::id::NodeId;
use hashbrown::HashMap;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

#[derive(Clone, Debug, Default)]
pub struct Payload {
    pub sensor_data: SensorData,
    pub total_size: f32,
    pub total_count: u32,
    pub downstream_data: Vec<NodeId>,
}

#[derive(Clone, Debug, Default)]
pub struct SensorData {
    pub node_info: NodeInfo,
    pub map_state: MapState,
    pub size_by_type: HashMap<DataType, f32>,
    pub count_by_type: HashMap<DataType, u32>,
}
