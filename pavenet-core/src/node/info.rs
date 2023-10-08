use pavenet_config::config::base::NodeType;
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::order::Order;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: u32,
    pub order: Order,
}
