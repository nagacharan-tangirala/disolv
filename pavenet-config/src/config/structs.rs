use crate::config::enums::NodeType;
use crate::types::ids::node::NodeId;
use crate::types::ids::road::RoadId;
use crate::types::order::Order;
use crate::types::velocity::Velocity;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct MapState {
    pub pos: Point2D,
    #[builder(default = None)]
    pub z: Option<f32>,
    #[builder(default = None)]
    pub velocity: Option<Velocity>,
    #[builder(default = None)]
    pub road_id: Option<RoadId>,
}

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct Link {
    pub target: Vec<NodeId>,
    #[builder(default = None)]
    pub distance: Option<Vec<f32>>,
    #[builder(default = None)]
    pub load_factor: Option<Vec<f32>>,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: u32,
    pub order: Order,
}
