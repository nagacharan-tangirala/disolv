use typed_builder::TypedBuilder;

use crate::net::message::{TxMetrics, V2XPayload};
use crate::net::slice::Slice;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Network {
    pub slices: Vec<Slice>,
}

impl Network {
    pub fn transfer(&mut self, payload: &V2XPayload) -> TxMetrics {
        self.slices
            .get_mut(0)
            .expect("no slice found")
            .transfer(payload)
    }

    pub fn reset_slices(&mut self) {
        self.slices.iter_mut().for_each(|slice| slice.reset());
    }
}
