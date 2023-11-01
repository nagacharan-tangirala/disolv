use crate::entity::class::NodeClass;
use crate::entity::kind::NodeType;
use crate::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::rules::{GTxRule, RuleAction, TxRuleEnforcer};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Copy, Default)]
#[serde(tag = "action_name", content = "towards")]
pub enum TxAction {
    #[default]
    Consume,
    ForwardToKind(NodeType),
    ForwardToTier(NodeClass),
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct RuleSettings {
    pub source: NodeClass,
    pub target: NodeClass,
    pub data_type: DataType,
    pub action: TxAction,
}

impl RuleAction<NodeClass> for TxAction {}

pub type DTxRule = GTxRule<DataType, TxAction, NodeClass>;

#[derive(Clone, Debug)]
pub struct Rules {
    pub tx_rules: HashMap<NodeClass, Vec<DTxRule>>,
}

impl Rules {
    pub fn new(rule_settings: Vec<RuleSettings>) -> Self {
        let mut tx_rules = HashMap::with_capacity(rule_settings.len());
        for r_set in rule_settings.into_iter() {
            let rule = DTxRule::builder()
                .data_source(r_set.source)
                .query_type(r_set.data_type)
                .action(r_set.action)
                .build();

            tx_rules
                .entry(r_set.target)
                .or_insert(Vec::new())
                .push(rule);
        }
        Self { tx_rules }
    }

    pub fn get_rule(&self, target: &NodeClass, source: &NodeClass) -> Option<Vec<&DTxRule>> {
        self.tx_rules.get(target).map(|rules| {
            rules
                .iter()
                .filter(|rule| rule.data_source == *source)
                .collect()
        })
    }
}

impl TxRuleEnforcer<NodeContent, PayloadInfo, DataType, TxAction, NodeClass> for Rules {
    fn enforce_tx_rules(&self, target: &NodeClass, mut payload: DPayload) -> DPayload {
        let source = &payload.content.node_info.node_class;
        let tx_rules: Vec<&DTxRule> = match self.get_rule(target, source) {
            Some(rules) => rules,
            None => return payload,
        };
        for rule in tx_rules.into_iter() {
            payload.metadata.apply_rule(rule);
        }
        payload
    }
}
