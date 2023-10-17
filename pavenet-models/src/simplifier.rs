use crate::model::{NodeModel, TomlReadable};
use crate::node::payload::Payload;
use serde::Deserialize;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct SimplifierSettings {
    pub name: String,
    pub compression_factor: Option<f32>,
    pub sampling_factor: Option<f32>,
}

impl TomlReadable for SimplifierSettings {}

#[derive(Clone, Debug, Copy)]
pub enum SimplifierType {
    Basic(BasicSimplifier),
    Random(RandomSimplifier),
}

impl SimplifierType {
    pub fn to_input(&self) -> SimplifierSettings {
        match self {
            SimplifierType::Basic(simplifier) => simplifier.to_input(),
            SimplifierType::Random(simplifier) => simplifier.to_input(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct BasicSimplifier {
    compression_factor: f32,
    sampling_factor: f32,
}

impl NodeModel for BasicSimplifier {
    type Input = SimplifierSettings;
    fn to_input(&self) -> SimplifierSettings {
        let name: String = "basic".to_string();
        SimplifierSettings {
            name,
            compression_factor: Some(self.compression_factor),
            sampling_factor: Some(self.sampling_factor),
        }
    }
}

impl BasicSimplifier {
    pub fn new(simplifier_settings: &SimplifierSettings) -> Self {
        let compression_factor = match simplifier_settings.compression_factor {
            Some(factor) => factor,
            None => 1.0,
        };
        let sampling_factor = match simplifier_settings.sampling_factor {
            Some(factor) => factor,
            None => 1.0,
        };
        Self {
            compression_factor,
            sampling_factor,
        }
    }

    pub(crate) fn simplify_payload(&self, payload: Payload) -> Payload {
        let mut simplified_payload = payload.clone();
        simplified_payload.sensor_data.size_by_type = payload
            .sensor_data
            .size_by_type
            .iter()
            .map(|(sensor_type, data_size)| (*sensor_type, data_size * self.compression_factor))
            .collect();
        simplified_payload.sensor_data.count_by_type = payload
            .sensor_data
            .count_by_type
            .iter()
            .map(|(sensor_type, data_count)| {
                (
                    *sensor_type,
                    (*data_count as f32 * self.sampling_factor) as u32,
                )
            })
            .collect();
        simplified_payload
    }
}

#[derive(Clone, Debug, Copy)]
pub struct RandomSimplifier {
    compression_factor: f32,
    sampling_factor: f32,
}

impl NodeModel for RandomSimplifier {
    type Input = SimplifierSettings;
    fn to_input(&self) -> SimplifierSettings {
        let name: String = "random".to_string();
        SimplifierSettings {
            name,
            compression_factor: Some(self.compression_factor),
            sampling_factor: Some(self.sampling_factor),
        }
    }
}

impl RandomSimplifier {
    pub fn new(simplifier_settings: &SimplifierSettings) -> Self {
        let compression_factor = match simplifier_settings.compression_factor {
            Some(factor) => factor,
            None => 1.0,
        };
        let sampling_factor = match simplifier_settings.sampling_factor {
            Some(factor) => factor,
            None => 1.0,
        };
        Self {
            compression_factor,
            sampling_factor,
        }
    }

    pub(crate) fn simplify_payload(&self, payload: Payload) -> Payload {
        payload
    }
}
