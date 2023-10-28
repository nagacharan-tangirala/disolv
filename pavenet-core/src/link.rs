use crate::node_info::id::NodeId;
use pavenet_engine::link::{Link, LinkFeatures};
use typed_builder::TypedBuilder;

pub type TLink = Link<LinkProperties, NodeId>;

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct LinkProperties {
    #[builder(default = None)]
    pub distance: Option<f32>,
    #[builder(default = None)]
    pub load_factor: Option<f32>,
}

impl LinkFeatures for LinkProperties {}
