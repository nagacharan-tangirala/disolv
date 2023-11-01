use crate::entity::Tier;
use crate::payload::{GPayload, PayloadContent, PayloadMetadata};
use crate::response::Queryable;
use typed_builder::TypedBuilder;

/// A trait that represents the type of a rule.
pub trait RuleAction<T>: Default + Copy + Clone + Send + Sync
where
    T: Tier,
{
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct GTxRule<Q, R, T>
where
    Q: Queryable,
    R: RuleAction<T>,
    T: Tier,
{
    pub data_source: T,
    pub query_type: Q,
    pub action: R,
}

impl<Q, R, T> GTxRule<Q, R, T>
where
    Q: Queryable,
    R: RuleAction<T>,
    T: Tier,
{
    pub fn new(data_source: T, query_type: Q, rule_type: R) -> Self {
        Self {
            data_source,
            query_type,
            action: rule_type,
        }
    }
}

pub trait TxRuleEnforcer<C, M, Q, R, T>: Clone + Send + Sync
where
    C: PayloadContent,
    M: PayloadMetadata<Q>,
    Q: Queryable,
    R: RuleAction<T>,
    T: Tier,
{
    fn enforce_tx_rules(&self, target: &T, payload: GPayload<C, M, Q>) -> GPayload<C, M, Q>;
}
