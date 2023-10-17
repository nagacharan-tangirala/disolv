use crate::node::id::NodeId;
use pavenet_core::node_finder::LinkInfo;
use serde_derive::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct Link {
    pub target: Vec<NodeId>,
    #[builder(default = None)]
    pub distance: Option<Vec<f32>>,
    #[builder(default = None)]
    pub load_factor: Option<Vec<f32>>,
}

impl LinkInfo<NodeId> for Link {}
