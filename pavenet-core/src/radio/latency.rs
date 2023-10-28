use crate::dist::{DistParams, RngSampler};
use crate::payload::PayloadInfo;
use crate::radio::metrics::latency::Latency;
use anyhow::Result;
use pavenet_engine::channel::{Measurable, MetricVariant, VariantConfig};
use serde::Deserialize;

/// All the latency configuration parameters are optional, but at least one of them must be present.
/// Name of the variant is mandatory.
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct LatencyConfig {
    pub variant: String,
    pub constant_term: Option<Latency>,
    pub min_latency: Option<Latency>,
    pub max_latency: Option<Latency>,
    pub factor: Option<f32>,
    pub dist_params: Option<DistParams>,
}

impl VariantConfig<Latency> for LatencyConfig {}

/// Random latency is sampled from a distribution of user's choice. Distribution parameters are
/// mandatory and must be valid for the chosen distribution.
#[derive(Debug, Clone, Copy)]
pub struct RandomLatency {
    pub min_latency: Latency,
    pub max_latency: Latency,
    pub sampler: RngSampler,
}

impl RandomLatency {
    pub fn new(min: Latency, max: Latency, dist_name: String, dist_params: DistParams) -> Self {
        Self {
            min_latency: min,
            max_latency: max,
            sampler: RngSampler::new(dist_name, dist_params),
        }
    }
}

impl Measurable<Latency, PayloadInfo> for RandomLatency {
    fn measure(&mut self, _payload: &PayloadInfo) -> Latency {
        let latency_factor = self.sampler.sample();
        let latency = self.min_latency + (self.max_latency - self.min_latency) * latency_factor;
    }
}

/// Distance latency is a linear function of the distance between the source and the destination.
/// The distance is taken from the selected link properties. The constant term and the factor are
/// mandatory. Latency is small for small distances and increases linearly with the distance.
/// Factor controls the slope of the linear function.
#[derive(Debug, Clone, Default, Copy)]
pub struct DistanceLatency {
    pub constant_term: Latency,
    pub factor: f32,
}

impl DistanceLatency {
    pub fn new(constant_term: Latency, factor: f32) -> Self {
        Self {
            constant_term,
            factor,
        }
    }
}

impl Measurable<Latency, PayloadInfo> for DistanceLatency {
    fn measure(&mut self, payload: &PayloadInfo) -> Latency {
        let distance_factor = match payload.selected_link.properties.distance {
            Some(distance) => distance * self.factor,
            None => 0.0,
        };
        return self.constant_term + Latency::from(distance_factor);
    }
}

/// Ordered latency is a linear function of the transmission order. The constant term and the
/// factor are mandatory. Latency is small for small order and increases linearly with the order.
/// Factor controls the slope of the linear function.
#[derive(Debug, Clone, Default, Copy)]
pub struct OrderedLatency {
    pub const_param: Latency,
    pub factor: f32,
}

impl OrderedLatency {
    pub fn new(const_param: Latency, factor: f32) -> Self {
        Self {
            const_param,
            factor,
        }
    }
}

impl Measurable<Latency, PayloadInfo> for OrderedLatency {
    fn measure(&mut self, payload: &PayloadInfo) -> Latency {
        let order_factor = match payload.tx_order {
            Some(distance) => distance * self.factor,
            None => 0.0,
        };
        return self.const_param + Latency::from(order_factor);
    }
}

/// Latency variant is a wrapper around all the possible latency variants. It is used to
/// instantiate the correct variant based on the configuration.
#[derive(Debug, Clone, Copy)]
pub enum LatencyVariant {
    Constant(Latency),
    Random(RandomLatency),
    Distance(DistanceLatency),
    Ordered(OrderedLatency),
}

impl LatencyVariant {
    fn build_constant(config: LatencyConfig) -> Result<Self> {
        let constant_latency = config
            .constant_term
            .ok_or(anyhow::anyhow!("Missing constant term"))?;
        Ok(Self::Constant(constant_latency))
    }

    fn build_random(config: LatencyConfig) -> Result<Self> {
        let min_latency = config
            .min_latency
            .ok_or(anyhow::anyhow!("Missing minimum latency"))?;
        let max_latency = config
            .max_latency
            .ok_or(anyhow::anyhow!("Missing maximum latency"))?;
        let dist_params = config
            .dist_params
            .ok_or(anyhow::anyhow!("Missing distribution parameters"))?;
        let dist_name = config
            .variant
            .ok_or(anyhow::anyhow!("Missing distribution name"))?;
        Ok(Self::Random(RandomLatency::new(
            min_latency,
            max_latency,
            dist_name,
            dist_params,
        )))
    }

    fn build_distance(config: LatencyConfig) -> Result<Self> {
        let constant_term = config
            .constant_term
            .ok_or(anyhow::anyhow!("Missing constant term"))?;
        let factor = config.factor.ok_or(anyhow::anyhow!("Missing factor"))?;
        Ok(Self::Distance(DistanceLatency::new(constant_term, factor)))
    }

    fn build_ordered(config: LatencyConfig) -> Result<Self> {
        let constant_term = config
            .constant_term
            .ok_or(anyhow::anyhow!("Missing constant term"))?;
        let factor = config.factor.ok_or(anyhow::anyhow!("Missing factor"))?;
        Ok(Self::Ordered(OrderedLatency::new(constant_term, factor)))
    }
}

impl MetricVariant<LatencyConfig, LatencyVariant, Latency> for LatencyVariant {
    fn new(variant_config: LatencyConfig) -> Self {
        match variant_config.variant.as_str() {
            "constant" => match Self::build_constant(variant_config) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid config for constant latency variant"),
            },
            "random" => match Self::build_random(variant_config) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid config for random latency variant"),
            },
            "distance" => match Self::build_distance(variant_config) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid config for distance latency variant"),
            },
            "ordered" => match Self::build_ordered(variant_config) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid config for ordered latency variant"),
            },
            _ => panic!("Invalid latency variant name"),
        }
    }
    fn measure(&mut self, payload: &PayloadInfo) -> Latency {
        match self {
            LatencyVariant::Constant(constant) => *constant,
            LatencyVariant::Random(sampler) => sampler.measure(payload),
            LatencyVariant::Distance(distance) => distance.measure(payload),
            LatencyVariant::Ordered(ordered) => ordered.measure(payload),
        }
    }
}
