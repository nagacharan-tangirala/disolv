use crate::entity::{Kind, Tier};
use crate::payload::{GPayload, PayloadContent, PayloadMetadata};
use crate::response::Queryable;
use hashbrown::HashMap;

/// A trait that represents the type of a rule.
pub trait RuleAction<K, T>: Default + Copy + Clone + Send + Sync
where
    K: Kind,
    T: Tier,
{
}

#[derive(Clone, Debug)]
pub struct GTxRule<K, Q, R, T>
where
    K: Kind,
    Q: Queryable,
    R: RuleAction<K, T>,
    T: Tier,
{
    pub data_source: K,
    pub query_type: Q,
    pub action: R,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<K, Q, R, T> GTxRule<K, Q, R, T>
where
    K: Kind,
    Q: Queryable,
    R: RuleAction<K, T>,
    T: Tier,
{
    pub fn new(data_source: K, query_type: Q, rule_type: R) -> Self {
        Self {
            data_source,
            query_type,
            action: rule_type,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn action_for(&self, query_type: &Q) -> R {
        if self.query_type == *query_type {
            self.action
        } else {
            R::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GTxRules<K, Q, R, T>
where
    K: Kind,
    Q: Queryable,
    R: RuleAction<K, T>,
    T: Tier,
{
    pub rules: HashMap<K, Vec<GTxRule<K, Q, R, T>>>, // target_kind -> rules
}

impl<K, Q, R, T> GTxRules<K, Q, R, T>
where
    K: Kind,
    Q: Queryable,
    R: RuleAction<K, T>,
    T: Tier,
{
    pub fn add_rule(&mut self, rule: GTxRule<K, Q, R, T>) {
        self.rules.entry(rule.data_source).or_default().push(rule);
    }

    pub fn get_rule(&self, target: &K, source: &K) -> Option<Vec<&GTxRule<K, Q, R, T>>> {
        self.rules.get(target).map(|rules| {
            rules
                .iter()
                .filter(|rule| rule.data_source == *source)
                .collect()
        })
    }
}

pub trait TxRuleEnforcer<C, K, M, Q, R, T>: Clone + Send + Sync
where
    C: PayloadContent<Q>,
    K: Kind,
    M: PayloadMetadata,
    Q: Queryable,
    R: RuleAction<K, T>,
    T: Tier,
{
    fn enforce_tx_rules(&self, target: &K, payload: GPayload<C, M, Q>) -> GPayload<C, M, Q>;
}
