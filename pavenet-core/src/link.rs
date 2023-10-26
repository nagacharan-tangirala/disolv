use crate::node_info::id::NodeId;
use pavenet_core::node_finder::LinkFeatures;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct Link {
    pub target: NodeId,
    pub properties: LinkProperties,
}

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct LinkProperties {
    #[builder(default = None)]
    pub distance: Option<f32>,
    #[builder(default = None)]
    pub load_factor: Option<f32>,
}

impl LinkFeatures for LinkProperties {}

impl Link {
    pub fn to_selected_link(&self, idx: usize) -> SelectedLink {
        let node_id = match self.target.get(idx) {
            Some(node_id) => *node_id,
            None => panic!("Invalid index: {}, length: {} ", idx, self.target.len()),
        };
        let distance = match self.distance {
            Some(ref distance) => distance.get(idx).map(|d| *d),
            None => None,
        };
        let load_factor = match self.load_factor {
            Some(ref load_factor) => load_factor.get(idx).map(|d| *d),
            None => None,
        };
        SelectedLink::new(node_id, distance, load_factor)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SelectedLink {
    pub node_id: NodeId,
    pub distance: Option<f32>,
    pub load_factor: Option<f32>,
}

impl SelectedLink {
    fn new(node_id: NodeId, distance: Option<f32>, load_factor: Option<f32>) -> Self {
        Self {
            node_id,
            distance,
            load_factor,
        }
    }
}
