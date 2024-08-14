use log::error;
use rand::RngCore;
use serde::Deserialize;

use disolv_core::message::{Metadata, TxReport};
use disolv_core::metrics::{Feasibility, Measurable, MetricSettings};
use disolv_core::radio::{Link, LinkFeatures};
use disolv_models::dist::{DistParams, RngSampler};
use disolv_models::net::metrics::Latency;

use crate::models::message::{PayloadInfo, TxMetrics};

/// All the latency configuration parameters are optional, but at least one of them must be present.
/// Name of the variant is mandatory.
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct LatencyConfig {
    pub variant: String,
    pub constraint: Latency,
    pub constant_term: Option<Latency>,
    pub min_latency: Option<Latency>,
    pub max_latency: Option<Latency>,
    pub factor: Option<f32>,
    pub dist_params: Option<DistParams>,
}

impl MetricSettings for LatencyConfig {}

/// Latency variant is a wrapper around all the possible latency variants. It is used to
/// instantiate the correct variant based on the configuration.
#[derive(Debug, Clone)]
pub enum LatencyType {
    Constant(ConstantLatency),
    Random(RandomLatency),
    Distance(DistanceLatency),
}

impl Measurable<Latency, PayloadInfo> for LatencyType {
    type S = LatencyConfig;

    fn with_settings(config: &LatencyConfig) -> Self {
        match config.variant.to_lowercase().as_str() {
            "constant" => LatencyType::Constant(ConstantLatency::with_settings(&config)),
            "random" => LatencyType::Random(RandomLatency::with_settings(&config)),
            "distance" => LatencyType::Distance(DistanceLatency::with_settings(&config)),
            _ => {
                error!("Only Constant, Random, and Distance latency variants are supported.");
                panic!("Unsupported latency variant {}.", config.variant);
            }
        }
    }

    fn measure(&mut self, metadata: &PayloadInfo) -> Feasibility<Latency> {
        match self {
            LatencyType::Constant(latency) => latency.measure(metadata),
            LatencyType::Random(latency) => latency.measure(metadata),
            LatencyType::Distance(latency) => latency.measure(metadata),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantLatency {
    pub latency: Latency,
}

impl Measurable<Latency, PayloadInfo> for ConstantLatency {
    type S = LatencyConfig;

    fn with_settings(config: &LatencyConfig) -> Self {
        let latency = config.constant_term.unwrap_or_else(|| {
            error!("Missing constant, setting it to 0.");
            Latency::new(0)
        });
        ConstantLatency { latency }
    }

    fn measure(&mut self, _metadata: &PayloadInfo) -> Feasibility<Latency> {
        Feasibility::Feasible(self.latency)
    }
}

/// Random latency is sampled from a distribution of user's choice. Distribution parameters are
/// mandatory and must be valid for the chosen distribution.
#[derive(Debug, Clone)]
pub struct RandomLatency {
    pub min_latency: Latency,
    pub max_latency: Latency,
    pub constraint: Latency,
    pub sampler: RngSampler,
}

impl Measurable<Latency, PayloadInfo> for RandomLatency {
    type S = LatencyConfig;

    fn with_settings(config: &LatencyConfig) -> Self {
        RandomLatency {
            min_latency: config.min_latency.expect("Missing min latency"),
            max_latency: config.max_latency.expect("Missing max latency"),
            constraint: config.constraint,
            sampler: RngSampler::new(
                config
                    .dist_params
                    .clone()
                    .expect("Missing distribution parameters"),
            ),
        }
    }

    fn measure(&mut self, _metadata: &PayloadInfo) -> Feasibility<Latency> {
        let latency_factor = self.sampler.rng.next_u64();
        let latency = Latency::new(5); // todo: Fix this
        if latency > self.constraint {
            return Feasibility::Infeasible(latency);
        }
        Feasibility::Feasible(latency)
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
    pub constraint: Latency,
}

impl Measurable<Latency, PayloadInfo> for DistanceLatency {
    type S = LatencyConfig;

    fn with_settings(config: &LatencyConfig) -> Self {
        DistanceLatency {
            constant_term: config.constant_term.unwrap_or(Latency::new(0)),
            factor: config.factor.unwrap_or(1.),
            constraint: config.constraint,
        }
    }

    fn measure(&mut self, metadata: &PayloadInfo) -> Feasibility<Latency> {
        let distance_factor = match metadata.selected_link.properties.distance {
            Some(distance) => distance * self.factor,
            None => 0.0,
        };
        let latency = Latency::new(self.constant_term.as_u64() + distance_factor as u64);
        if latency > self.constraint {
            return Feasibility::Infeasible(latency);
        }
        Feasibility::Feasible(latency)
    }
}
