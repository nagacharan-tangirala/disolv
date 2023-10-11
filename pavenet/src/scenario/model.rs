use crate::scenario::episode::ModelChanges;
use pavenet_models::node::composer::{
    BasicComposer, ComposerSettings, ComposerType, StatusComposer,
};
use pavenet_models::node::responder::ResponderType;
use pavenet_models::node::simplifier::{
    BasicSimplifier, RandomSimplifier, SimplifierSettings, SimplifierType,
};

#[derive(Debug, Copy, Clone)]
pub struct DeviceModel {
    pub composer: Option<ComposerType>,
    pub simplifier: Option<SimplifierType>,
    pub responder: Option<ResponderType>,
}

impl DeviceModel {
    pub fn new() -> ModelBuilder {
        ModelBuilder::new()
    }

    pub fn fetch_current_settings(&mut self, new_models: &ModelChanges) -> ModelChanges {
        let mut current_models = ModelChanges::default();
        current_models.composer = match new_models.composer {
            Some(ref composer) => composer.to_input(),
            None => None,
        };
        current_models.responder = match new_models.responder {
            Some(ref responder) => responder.to_input(),
            None => None,
        };
        current_models.simplifier = match new_models.simplifier {
            Some(ref simplifier) => simplifier.to_input(),
            None => None,
        };
        return current_models;
    }

    pub fn update_models(&mut self, models: &ModelChanges) {}
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ModelBuilder {
    pub composer: Option<ComposerType>,
    pub simplifier: Option<SimplifierType>,
    pub responder: Option<ResponderType>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            composer: None,
            simplifier: None,
            responder: None,
        }
    }

    pub fn with_composer(mut self, composer_settings: &Option<ComposerSettings>) -> Self {
        self.composer = match composer_settings {
            Some(ref composer_settings) => match composer_settings.name.as_str() {
                "basic" => Some(ComposerType::Basic(BasicComposer::new(composer_settings))),
                "status" => Some(ComposerType::Status(StatusComposer::new(composer_settings))),
                _ => panic!("Unknown composer type"),
            },
            None => None,
        };
        self
    }

    pub fn with_simplifier(mut self, simplifier: &Option<SimplifierSettings>) -> Self {
        self.simplifier = match simplifier {
            Some(ref simplifier_settings) => match simplifier_settings.name.as_str() {
                "basic" => Some(SimplifierType::Basic(BasicSimplifier::new(
                    simplifier_settings,
                ))),
                "random" => Some(SimplifierType::Random(RandomSimplifier::new(
                    simplifier_settings,
                ))),
                _ => panic!("Unknown simplifier type"),
            },
            None => None,
        };
        self
    }

    pub fn build(self) -> DeviceModel {
        DeviceModel {
            composer: self.composer,
            simplifier: self.simplifier,
            responder: self.responder,
        }
    }
}
