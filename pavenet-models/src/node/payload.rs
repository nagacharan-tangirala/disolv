use hashbrown::HashMap;
use pavenet_core::enums::DataType;
use pavenet_core::structs::{MapState, NodeInfo};
use pavenet_core::types::NodeId;

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
