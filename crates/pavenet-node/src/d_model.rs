use pavenet_core::power::PowerManager;
use pavenet_input::power::data::PowerTimes;
use pavenet_models::compose::{BasicComposer, Composer, ComposerSettings, StatusComposer};
use pavenet_models::radio::Radio;
use pavenet_models::respond::{Responder, ResponderSettings, StatsResponder};
use pavenet_models::select::{Selector, SelectorSettings};

#[derive(Debug, Clone)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub radio: Radio,
    pub composer: Option<Composer>,
    pub responder: Option<Responder>,
    pub selector: Option<Selector>,
}

impl DeviceModel {
    pub fn builder(radio: Radio) -> ModelBuilder {
        ModelBuilder::builder(radio)
    }
}

#[derive(Debug, Clone)]
pub struct ModelBuilder {
    pub power: PowerManager,
    pub radio: Radio,
    pub composer: Option<Composer>,
    pub responder: Option<Responder>,
    pub selector: Option<Selector>,
}

impl ModelBuilder {
    pub fn builder(radio: Radio) -> Self {
        Self {
            power: Default::default(),
            radio,
            composer: None,
            responder: None,
            selector: None,
        }
    }

    pub fn with_composer(mut self, composer_settings: Option<ComposerSettings>) -> Self {
        match composer_settings {
            Some(settings) => {
                self.composer = Some(match settings.name.as_str() {
                    "basic" => Composer::Basic(BasicComposer::new(&settings)),
                    "status" => Composer::Status(StatusComposer::new(&settings)),
                    _ => panic!("Unknown composer type"),
                });
            }
            None => {}
        };
        self
    }

    pub fn with_responder(mut self, responder_settings: Option<ResponderSettings>) -> Self {
        match responder_settings {
            Some(settings) => {
                self.responder = Some(match settings.name.as_str() {
                    "stats" => Responder::Stats(StatsResponder::new(settings.clone())),
                    _ => panic!("Unknown responder type"),
                });
            }
            None => {}
        };
        self
    }

    pub fn with_selector(mut self, selector_settings: Option<SelectorSettings>) -> Self {
        match selector_settings {
            Some(settings) => self.selector = Some(Selector::new(settings.clone())),
            None => {}
        };
        self
    }

    pub fn with_power(mut self, power_times: PowerTimes) -> Self {
        self.power = PowerManager::new(power_times);
        self
    }

    pub fn build(self) -> DeviceModel {
        DeviceModel {
            composer: self.composer,
            responder: self.responder,
            selector: self.selector,
            radio: self.radio,
            power: self.power,
        }
    }
}
