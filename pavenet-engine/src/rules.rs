use crate::entity::Tier;
use crate::payload::{GPayload, PayloadContent, PayloadMetadata};
use crate::response::Queryable;
use hashbrown::HashMap;

/// A trait that represents the type of a rule.
pub trait RuleAction<T>: Default + Copy + Clone + Send + Sync
where
    T: Tier,
{
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct GTxRules<Q, R, T>
where
    Q: Queryable,
    R: RuleAction<T>,
    T: Tier,
{
    pub rules: HashMap<T, Vec<GTxRule<Q, R, T>>>, // target_kind -> rules
}

impl<Q, R, T> GTxRules<Q, R, T>
where
    Q: Queryable,
    R: RuleAction<T>,
    T: Tier,
{
    pub fn add_rule(&mut self, rule: GTxRule<Q, R, T>) {
        self.rules.entry(rule.data_source).or_default().push(rule);
    }

    pub fn get_rule(&self, target: &T, source: &T) -> Option<Vec<&GTxRule<Q, R, T>>> {
        self.rules.get(target).map(|rules| {
            rules
                .iter()
                .filter(|rule| rule.data_source == *source)
                .collect()
        })
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
