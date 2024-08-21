use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;

use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

use crate::models::ai::models::ModelType;

pub(crate) struct ModelLake<A: AutodiffBackend, B: Backend> {
    pub(crate) model_map: HashMap<AgentId, ModelType<A, B>>,
}

impl<A: AutodiffBackend, B: Backend> ModelLake<A, B> {
    pub(crate) fn add_local_model(&mut self, agent_id: AgentId, model: ModelType<A, B>) {
        self.model_map.insert(agent_id, model);
    }

    pub(crate) fn local_model_of(&mut self, agent_id: AgentId) -> ModelType<A, B> {
        self.model_map
            .remove(&agent_id)
            .expect("failed to find local model")
    }

    pub(crate) fn local_models(&mut self) -> Vec<ModelType<A, B>> {
        let local_models = self.model_map.values().collect();
        self.model_map.clear();
        local_models
    }
}
