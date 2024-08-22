use disolv_core::agent::AgentClass;
use disolv_core::hashbrown::HashMap;
use disolv_core::message::ContentType;
use disolv_core::radio::Action;

use crate::net::radio::ActionSettings;

#[derive(Clone, Debug, Default)]
pub struct Actor<C: ContentType> {
    pub actions: HashMap<AgentClass, HashMap<C, Action>>,
}

impl<C: ContentType> Actor<C> {
    pub fn new(action_settings: &Option<Vec<ActionSettings<C>>>) -> Self {
        let action_settings = match action_settings {
            Some(settings) => settings,
            None => return Self::default(),
        };
        let mut actions: HashMap<AgentClass, HashMap<C, Action>> = HashMap::new();

        for action_setting in action_settings.iter() {
            let action = Action::builder()
                .action_type(action_setting.action_type)
                .to_kind(action_setting.to_kind)
                .to_class(action_setting.to_class)
                .to_agent(action_setting.to_agent)
                .to_broadcast(action_setting.to_broadcast.clone())
                .build();

            actions
                .entry(action_setting.target)
                .or_default()
                .entry(action_setting.data_type)
                .or_insert(action);
        }
        Actor { actions }
    }

    pub fn actions_for(&self, target_class: &AgentClass) -> &HashMap<C, Action> {
        self.actions
            .get(target_class)
            .expect("Missing actions for class")
    }
}
