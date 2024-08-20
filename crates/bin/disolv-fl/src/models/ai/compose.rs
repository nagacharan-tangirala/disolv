use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::hashbrown::HashMap;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::Action;

use crate::models::device::message::{Message, MessageType, MessageUnit};

#[derive(Debug, Clone, Deserialize)]
pub struct FlDataSettings {
    pub size_map: HashMap<Message, Bytes>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FlComposerSettings {
    pub name: String,
    pub data_settings: FlDataSettings,
}

impl ModelSettings for FlComposerSettings {}

#[derive(Debug, Clone)]
pub enum FlComposer {
    Simple(SimpleComposer),
}

impl FlComposer {
    pub(crate) fn compose_data_unit(&mut self, message: FlMessageToBuild) -> MessageUnit {
        match self {
            FlComposer::Simple(composer) => composer.compose_data_unit(message),
        }
    }
}

impl Model for FlComposer {
    type Settings = FlComposerSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "simple" => FlComposer::Simple(SimpleComposer::new(settings)),
            _ => panic!("Unsupported composer type {}.", settings.name),
        }
    }
}

#[derive(TypedBuilder)]
pub struct FlMessageToBuild {
    pub message: Message,
    pub message_type: MessageType,
    pub quantity: u64,
}

#[derive(Debug, Clone)]
pub struct SimpleComposer {
    pub settings: FlDataSettings,
}

impl SimpleComposer {
    fn new(composer_settings: &FlComposerSettings) -> Self {
        Self {
            settings: composer_settings.data_settings.to_owned(),
        }
    }

    pub(crate) fn compose_data_unit(&self, message_info: FlMessageToBuild) -> MessageUnit {
        let mut message_size = self
            .settings
            .size_map
            .get(&message_info.message)
            .expect("Invalid message")
            .to_owned();

        message_size = Bytes::new(message_size.as_u64() * message_info.quantity);

        MessageUnit::builder()
            .message_type(message_info.message_type.clone())
            .message_size(message_size)
            .action(Action::default())
            .build()
    }
}
