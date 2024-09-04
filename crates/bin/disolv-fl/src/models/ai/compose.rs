use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::hashbrown::HashMap;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};

use crate::fl::client::AgentInfo;
use crate::models::device::message::{
    FlContent, FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit,
};

#[derive(Debug, Clone, Deserialize)]
pub struct FlDataSettings {
    pub size_map: HashMap<FlContent, Bytes>,
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
    pub(crate) fn compose_payload(&mut self, agent_state: &AgentInfo) -> FlPayload {
        match self {
            FlComposer::Simple(composer) => composer.compose_payload(agent_state),
        }
    }

    pub(crate) fn set_message_to_build(&mut self, message: FlMessageToBuild) {
        match self {
            FlComposer::Simple(composer) => composer.set_message_to_build(message),
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

#[derive(Clone, Default, Copy, Debug, TypedBuilder)]
pub struct FlMessageToBuild {
    pub message: Message,
    pub message_type: MessageType,
    pub fl_content: FlContent,
    pub quantity: u64,
}

#[derive(Debug, Clone)]
pub struct SimpleComposer {
    pub settings: FlDataSettings,
    pub message_to_send: FlMessageToBuild,
}

impl SimpleComposer {
    fn new(composer_settings: &FlComposerSettings) -> Self {
        Self {
            settings: composer_settings.data_settings.to_owned(),
            message_to_send: FlMessageToBuild::default(),
        }
    }

    fn set_message_to_build(&mut self, message: FlMessageToBuild) {
        self.message_to_send = message;
    }

    fn compose_payload(&self, agent_state: &AgentInfo) -> FlPayload {
        let message_unit = self.compose_data_unit();
        let payload_info = self.build_metadata(&message_unit);
        FlPayload::builder()
            .agent_state(agent_state.clone())
            .data_units(vec![message_unit])
            .query_type(Message::FlMessage)
            .gathered_states(Vec::new())
            .metadata(payload_info)
            .build()
    }

    fn compose_data_unit(&self) -> MessageUnit {
        let mut message_size = self
            .settings
            .size_map
            .get(&self.message_to_send.fl_content)
            .expect("Invalid message")
            .to_owned();

        message_size = Bytes::new(message_size.as_u64() * self.message_to_send.quantity);
        MessageUnit::builder()
            .message_type(self.message_to_send.message_type.clone())
            .message_size(message_size)
            .action(Action::default())
            .fl_content(self.message_to_send.fl_content)
            .build()
    }

    fn build_metadata(&self, data_unit: &MessageUnit) -> FlPayloadInfo {
        let payload_info = FlPayloadInfo::builder()
            .total_size(data_unit.message_size)
            .total_count(1)
            .selected_link(Link::default())
            .build();
        payload_info
    }
}
