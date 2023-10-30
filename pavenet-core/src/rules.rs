use crate::entity::class::NodeClass;
use crate::entity::kind::NodeType;
use crate::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_engine::rules::{GTxRule, GTxRules, RuleAction, TxRuleEnforcer};

#[derive(Clone, Debug, Copy, Default)]
pub enum Actions {
    #[default]
    Consume,
    ForwardToKind(NodeType),
    ForwardToTier(NodeClass),
}

impl RuleAction<NodeClass> for Actions {}

pub type DTxRule = GTxRule<DataType, Actions, NodeClass>;
pub type DTxRules = GTxRules<DataType, Actions, NodeClass>;

#[derive(Clone, Debug)]
pub struct Rules {
    pub tx_rules: DTxRules,
}

impl Rules {
    pub fn new(tx_rules: DTxRules) -> Self {
        Self { tx_rules }
    }
}

impl TxRuleEnforcer<NodeContent, PayloadInfo, DataType, Actions, NodeClass> for Rules {
    fn enforce_tx_rules(&self, target: &NodeClass, mut payload: DPayload) -> DPayload {
        let source = &payload.content.node_info.node_class;
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
