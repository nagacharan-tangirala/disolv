use crate::entity::Identifier;
use std::fmt::Debug;
use typed_builder::TypedBuilder;

/// A trait that contains information about a link. It could be distance, load, etc.
pub trait LinkFeatures: Clone + Debug + Default {}

/// A struct that represents a link between two nodes defined by the features F.
#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct GLink<F, I>
where
    F: LinkFeatures,
    I: Identifier,
{
    pub target: I,
    pub properties: F,
}

impl<F, I> GLink<F, I>
where
    F: LinkFeatures,
    I: Identifier,
{
    pub fn new(target: I, properties: F) -> Self {
        Self { target, properties }
    }
}

/// A struct that represents a set of links between a node and other nodes at a given time.
/// User can write strategies to select the best link to use.
#[derive(Debug, Clone, Default)]
pub struct GLinkOptions<F, I>
where
    F: LinkFeatures,
    I: Identifier,
{
    pub links: Vec<GLink<F, I>>,
}

impl<F, I> GLinkOptions<F, I>
where
    F: LinkFeatures,
    I: Identifier,
{
    pub fn new(targets: Vec<I>, properties: F) -> Self {
        let links = targets
            .into_iter()
            .map(|target| GLink::new(target, properties.clone()))
            .collect();
        Self { links }
    }
}