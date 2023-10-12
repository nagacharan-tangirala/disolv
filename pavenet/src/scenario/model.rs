use crate::scenario::episode::ModelChanges;
use pavenet_models::node::composer::{
    BasicComposer, ComposerSettings, ComposerType, StatusComposer,
};
use pavenet_models::node::responder::ResponderType;
use pavenet_models::node::simplifier::{
    BasicSimplifier, RandomSimplifier, SimplifierSettings, SimplifierType,
};

#[derive(Debug, Copy, Clone, Default)]
pub struct DeviceModel {
    pub composer: Option<ComposerType>,
    pub simplifier: Option<SimplifierType>,
    pub responder: Option<ResponderType>,
}

impl DeviceModel {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::builder()
    }

    pub fn fetch_current_settings(&mut self, models_to_change: &ModelChanges) -> ModelChanges {
        let mut current_models = ModelChanges::default();
        current_models.composer = match models_to_change.composer {
            Some(_) => self.composer.map(|composer| composer.to_input()),
            None => None,
        };
        current_models.simplifier = match models_to_change.simplifier {
            Some(_) => self.simplifier.map(|simplifier| simplifier.to_input()),
            None => None,
        };
        current_models.responder = match models_to_change.responder {
            Some(_) => self.responder.map(|responder| responder.to_input()),
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
    pub fn builder() -> Self {
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
                "simple" => Some(SimplifierType::Basic(BasicSimplifier::new(
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
