use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::hashbrown::HashMap;
use disolv_core::metrics::Bytes;
use disolv_core::model::{Model, ModelSettings};
use disolv_core::radio::{Action, Link};

use crate::fl::client::AgentInfo;
use crate::models::device::message::{FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit};

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
    pub(crate) fn compose_payload(
        &mut self,
        agent_state: AgentInfo,
        message: FlMessageToBuild,
    ) -> FlPayload {
        match self {
            FlComposer::Simple(composer) => composer.compose_payload(agent_state, message),
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

    pub(crate) fn compose_payload(
        &self,
        agent_state: AgentInfo,
        message_to_build: FlMessageToBuild,
    ) -> FlPayload {
        let message_unit = self.compose_data_unit(message_to_build);
        let payload_info = self.build_metadata(&message_unit);
        FlPayload::builder()
            .agent_state(agent_state)
            .data_units(vec![message_unit])
            .query_type(Message::StateInfo)
            .gathered_states(None)
            .metadata(payload_info)
            .build()
    }

    fn compose_data_unit(&self, message_info: FlMessageToBuild) -> MessageUnit {
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

    fn build_metadata(&self, data_unit: &MessageUnit) -> FlPayloadInfo {
        let payload_info = FlPayloadInfo::builder()
            .total_size(data_unit.message_size)
            .total_count(1)
            .selected_link(Link::default())
            .build();
        payload_info
    }
}
