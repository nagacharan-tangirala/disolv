use crate::entity::kind::NodeType;
use crate::entity::order::Order;
use crate::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_engine::rules::{GTxRule, GTxRules, RuleAction, TxRuleEnforcer};

#[derive(Clone, Debug, Copy, Default)]
pub enum Actions {
    #[default]
    Consume,
    ForwardToKind(NodeType),
    ForwardToTier(Order),
}

impl RuleAction<NodeType, Order> for Actions {}

pub type DTxRule = GTxRule<NodeType, DataType, Actions, Order>;
pub type DTxRules = GTxRules<NodeType, DataType, Actions, Order>;

#[derive(Clone, Debug)]
pub struct Rules {
    pub tx_rules: DTxRules,
}

impl Rules {
    pub fn new(tx_rules: DTxRules) -> Self {
        Self { tx_rules }
    }
}

impl TxRuleEnforcer<NodeContent, NodeType, PayloadInfo, DataType, Actions, Order> for Rules {
    fn enforce_tx_rules(&self, target: &NodeType, mut payload: DPayload) -> DPayload {
        let source = &payload.content.node_info.node_type;
        let tx_rules: Vec<&DTxRule> = match self.tx_rules.get_rule(target, source) {
            Some(rules) => rules,
            None => return payload,
        };
        for rule in tx_rules.into_iter() {
            payload.metadata.apply_rule(rule);
        }
        payload
    }
}
