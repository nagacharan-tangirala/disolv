use pavenet_engine::link::{GLink, LinkFeatures};
use typed_builder::TypedBuilder;

#[derive(Debug, Copy, Clone, Default, TypedBuilder)]
pub struct LinkProperties {
    #[builder(default = None)]
    pub distance: Option<f32>,
    #[builder(default = None)]
    pub load_factor: Option<f32>,
}

impl LinkFeatures for LinkProperties {}

pub type DLink = GLink<LinkProperties>;
