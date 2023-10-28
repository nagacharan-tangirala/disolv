use std::fmt::Debug;

/// A trait that contains information about a link. It could be distance, load, etc.
pub trait LinkFeatures: Clone + Debug + Default {}

/// A struct that represents a link between two nodes defined by the features F.
#[derive(Debug, Clone, Default)]
pub struct Link<F, I> {
    pub target: I,
    pub properties: F,
}

impl<F, I> Link<F, I> {
    pub fn new(target: I, properties: F) -> Self {
        Self { target, properties }
    }
}

/// A struct that represents a set of links between a node and other nodes at a given time.
/// User can write strategies to select the best link to use.
#[derive(Debug, Clone, Default)]
pub struct LinkOptions<F, I> {
    pub links: Vec<Link<F, I>>,
}

impl<F, I> LinkOptions<F, I> {
    pub fn new(targets: Vec<I>, properties: F) -> Self {
        let links = targets
            .into_iter()
            .map(|target| Link::new(target, properties.clone()))
            .collect();
        Self { links }
    }
}
