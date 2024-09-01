use burn::prelude::Backend;

use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

use crate::models::ai::models::ModelType;

pub(crate) struct ModelLake<B: Backend> {
    pub(crate) global_model: Option<ModelType<B>>,
    pub(crate) model_map: HashMap<AgentId, ModelType<B>>,
}

impl<B: Backend> ModelLake<B> {
    pub(crate) fn new() -> Self {
        Self {
            global_model: None,
            model_map: HashMap::new(),
        }
    }

    pub(crate) fn update_global_model(&mut self, new_model: ModelType<B>) {
        self.global_model = Some(new_model);
    }

    pub(crate) fn add_local_model(&mut self, agent_id: AgentId, model: ModelType<B>) {
        self.model_map.insert(agent_id, model);
    }

    pub(crate) fn local_model_of(&mut self, agent_id: AgentId) -> ModelType<B> {
        self.model_map
            .remove(&agent_id)
            .expect("failed to find local model")
    }

    pub(crate) fn local_models(&mut self) -> Vec<ModelType<B>> {
        let local_models = self.model_map.clone().into_values().collect();
        self.model_map.clear();
        local_models
    }
}
