use crate::models::compose::Composer;
use crate::models::power::PowerManager;
use crate::models::radio::Radio;
use crate::models::respond::Responder;
use crate::models::select::Selector;
use pavenet_core::bucket::TimeS;
use typed_builder::TypedBuilder;

pub trait BucketModel {
    fn init(&mut self, step: TimeS);
    fn stream_data(&mut self, step: TimeS);
    fn refresh_cache(&mut self, step: TimeS);
}

#[derive(Debug, Clone)]
pub struct DeviceModel {
    pub composer: Composer,
    pub responder: Responder,
    pub selector: Selector,
    pub channel: Radio,
    pub power: PowerManager,
}

impl DeviceModel {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::builder()
    }
}

#[derive(Debug, Clone)]
pub struct ModelBuilder {
    pub composer: Composer,
    pub responder: Responder,
    pub selector: Selector,
    pub channel: Radio,
    pub power: PowerManager,
}

impl ModelBuilder {
    pub fn builder() -> Self {
        Self {
            composer: Composer::Basic(BasicComposer::default()),
            responder: Responder::Stats(StatsResponder::default()),
            selector: Selector::default(),
        }
    }

    pub fn with_composer(mut self, composer_settings: &ComposerSettings) -> Self {
        match composer_settings.name.as_str() {
            "basic" => Composer::Basic(BasicComposer::new(composer_settings.clone())),
            "status" => Composer::Status(StatusComposer::new(composer_settings.clone())),
            _ => panic!("Unknown composer type"),
        };
        self
    }

    pub fn with_responder(mut self, responder_settings: &ResponderSettings) -> Self {
        match responder_settings.name.as_str() {
            "basic" => Responder::Stats(StatsResponder::new(responder_settings.clone())),
            _ => panic!("Unknown responder type"),
        };
        self
    }

    pub fn with_selector(mut self, selector_settings: &SelectorSettings) -> Self {
        match selector_settings.strategy {};
        self
    }

    pub fn with_channel(mut self, channel: Radio) -> Self {
        self.channel = channel;
        self
    }

    pub fn with_power(mut self, power: PowerManager) -> Self {
        self.power = power;
        self
    }

    pub fn build(self) -> DeviceModel {
        DeviceModel {
            composer: self.composer,
            responder: self.responder,
            selector: self.selector,
            channel: Default::default(),
            power: Default::default(),
        }
    }
}
