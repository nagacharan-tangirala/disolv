use crate::device::types::DeviceClass;
use crate::net::radio::{Action, ActionSettings, DActions};

#[derive(Clone, Debug, Default)]
pub struct Actor {
    pub target_classes: Vec<DeviceClass>,
    pub actions: Vec<(DeviceClass, DActions)>,
}

impl Actor {
    pub fn new(action_settings: &Option<Vec<ActionSettings>>) -> Self {
        let action_settings = match action_settings {
            Some(settings) => settings,
            None => return Self::default(),
        };
        let mut actions: Vec<(DeviceClass, DActions)> = Vec::new();
        let mut target_classes: Vec<DeviceClass> = Vec::new();

        for action_setting in action_settings.iter() {
            let action = Action::builder()
                .action_type(action_setting.action_type)
                .to_kind(action_setting.to_kind)
                .to_class(action_setting.to_class)
                .to_node(action_setting.to_node)
                .build();

            if let Some(class_actions) = actions.iter_mut().find(|x| x.0 == action_setting.target) {
                class_actions.1.add_action(action_setting.data_type, action);
            } else {
                let mut new_action = DActions::default();
                new_action.add_action(action_setting.data_type, action);
                actions.push((action_setting.target, new_action));
            }
            if !target_classes.contains(&action_setting.target) {
                target_classes.push(action_setting.target);
            }
        }
        Actor {
            actions,
            target_classes,
        }
    }

    pub fn actions_for(&self, target_class: &DeviceClass) -> &DActions {
        &self
            .actions
            .iter()
            .find(|x| x.0 == *target_class)
            .expect("Missing actions for class")
            .1
    }

    pub fn apply_actions(&self, target_class: &DeviceClass) {}
}
