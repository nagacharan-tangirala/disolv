use crate::enums::NodeType;
use crate::named::class::Class;
use crate::named::ids::node::NodeId;
use crate::named::ids::road::RoadId;
use crate::named::order::Order;
use crate::named::velocity::Velocity;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
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

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: Class,
    pub order: Order,
}
