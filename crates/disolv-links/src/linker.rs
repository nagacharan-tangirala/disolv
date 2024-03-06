use crate::reader::AgentIdPos;
use kiddo::{KdTree, NearestNeighbour, SquaredEuclidean};
use serde::Deserialize;

#[derive(Copy, Clone, Default, Debug, Deserialize)]
pub struct Radius(f64);

impl From<f64> for Radius {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Radius {
    pub(crate) fn as_f64(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, Default, Debug, Deserialize)]
pub struct DeviceCount(u32);

impl From<u32> for DeviceCount {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl DeviceCount {
    pub(crate) fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum LinkModel {
    Circular,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) enum LinkType {
    Static,
    Dynamic,
}
