use advaitars_core::message::{DPayload, TxMetrics};
use advaitars_models::slice::Slice;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Network {
    pub(crate) slices: Vec<Slice>,
}

impl Network {
    pub fn transfer(&mut self, payload: &DPayload) -> TxMetrics {
        self.slices
            .get_mut(0)
            .expect("no slice found")
            .transfer(payload)
    }

    pub fn reset_slices(&mut self) {
        self.slices.iter_mut().for_each(|slice| slice.reset());
    }
}
