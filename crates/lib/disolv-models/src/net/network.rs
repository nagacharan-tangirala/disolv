use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use typed_builder::TypedBuilder;

use disolv_core::agent::AgentProperties;
use disolv_core::hashbrown::HashMap;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType, TxReport};

/// Mark an enum with this trait to contain the various slice type configurations.
pub trait SliceType: Default + Copy + Clone + Eq + Hash + Debug {}

/// Every network slice should implement this trait.
pub trait NetworkSlice<C, D, M, P, Q, T>: Clone + Debug
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
    T: TxReport,
{
    fn reset(&mut self);
    fn transfer(&mut self, payload: &Payload<C, D, M, P, Q>) -> T;
}

/// A generic struct containing slices and their respective types.
#[derive(Clone, Debug, TypedBuilder)]
pub struct Network<N, C, D, M, P, Q, S, T>
where
    N: NetworkSlice<C, D, M, P, Q, T>,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
    S: SliceType,
    T: TxReport,
{
    pub slices: HashMap<S, N>,
    #[builder(default)]
    phantom_data: PhantomData<fn() -> (C, D, M, P, Q, T)>,
}

impl<N, C, D, M, P, Q, S, T> Network<N, C, D, M, P, Q, S, T>
where
    N: NetworkSlice<C, D, M, P, Q, T>,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
    S: SliceType,
    T: TxReport,
{
    pub fn transfer(&mut self, payload: &Payload<C, D, M, P, Q>) -> T {
        self.slices
            .get_mut(&S::default())
            .expect("Missing slice of default type")
            .transfer(payload)
    }

    pub fn slice_of_type(&mut self, slice_type: &S) -> &N {
        self.slices.get(slice_type).expect("Invalid type requested")
    }

    pub fn reset_slices(&mut self) {
        self.slices.values_mut().for_each(|slice| slice.reset());
    }
}
