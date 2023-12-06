use log::error;
use pavenet_core::dist::{DistParams, RngSampler};
use pavenet_core::message::PayloadInfo;
use pavenet_core::metrics::Latency;
use pavenet_engine::metrics::{Feasibility, Measurable, Metric, MetricSettings};
use serde::Deserialize;

/// All the latency configuration parameters are optional, but at least one of them must be present.
/// Name of the variant is mandatory.
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct LatencyConfig {
    pub variant: String,
    pub constraint: Latency,
    pub constant_term: Option<Latency>,
    pub min_latency: Option<f32>,
    pub max_latency: Option<f32>,
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
    Ordered(OrderedLatency),
}

impl Measurable<Latency> for LatencyType {
    type P = PayloadInfo;
    type S = LatencyConfig;

    fn with_settings(config: LatencyConfig) -> Self {
        match config.variant.to_lowercase().as_str() {
            "constant" => LatencyType::Constant(ConstantLatency::with_settings(config)),
            "random" => LatencyType::Random(RandomLatency::with_settings(config)),
            "distance" => LatencyType::Distance(DistanceLatency::with_settings(config)),
            "ordered" => LatencyType::Ordered(OrderedLatency::with_settings(config)),
            _ => {
                error!(
                    "Only Constant, Random, Distance and Ordered latency variants are supported."
                );
                panic!("Unsupported latency variant {}.", config.variant);
            }
        }
    }

    fn measure(&mut self, payload: &PayloadInfo) -> Feasibility<Latency> {
        match self {
            LatencyType::Constant(latency) => latency.measure(payload),
            LatencyType::Random(latency) => latency.measure(payload),
            LatencyType::Distance(latency) => latency.measure(payload),
            LatencyType::Ordered(latency) => latency.measure(payload),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantLatency {
    pub latency: Latency,
}

impl Measurable<Latency> for ConstantLatency {
    type P = PayloadInfo;
    type S = LatencyConfig;

    fn with_settings(config: LatencyConfig) -> Self {
        let latency = match config.constant_term {
            Some(latency) => latency,
            None => {
                error!("Missing constant, setting it to 0.");
                Latency::from(0.)
            }
        };
        ConstantLatency { latency }
    }

    fn measure(&mut self, _payload: &PayloadInfo) -> Feasibility<Latency> {
        Feasibility::Feasible(self.latency)
    }
}

/// Random latency is sampled from a distribution of user's choice. Distribution parameters are
/// mandatory and must be valid for the chosen distribution.
#[derive(Debug, Clone)]
pub struct RandomLatency {
    pub min_latency: f32,
    pub max_latency: f32,
    pub constraint: Latency,
    pub sampler: RngSampler,
}

impl Measurable<Latency> for RandomLatency {
    type P = PayloadInfo;
    type S = LatencyConfig;

    fn with_settings(config: LatencyConfig) -> Self {
        RandomLatency {
            min_latency: config.min_latency.expect("Missing min latency"),
            max_latency: config.max_latency.expect("Missing max latency"),
            constraint: config.constraint,
            sampler: RngSampler::new(config.dist_params.expect("Missing distribution parameters")),
        }
    }

    fn measure(&mut self, _payload: &PayloadInfo) -> Feasibility<Latency> {
        let latency_factor = self.sampler.sample();
        let lt_f32 = self.min_latency + (self.max_latency - self.min_latency) * latency_factor;
        let latency = Latency::from(lt_f32);
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

impl Measurable<Latency> for DistanceLatency {
    type P = PayloadInfo;
    type S = LatencyConfig;

    fn with_settings(config: LatencyConfig) -> Self {
        DistanceLatency {
            constant_term: config.constant_term.unwrap_or(Latency::from(0.)),
            factor: config.factor.unwrap_or(1.),
            constraint: config.constraint,
        }
    }

    fn measure(&mut self, payload: &PayloadInfo) -> Feasibility<Latency> {
        let distance_factor = match payload.routing_info.selected_link.properties.distance {
            Some(distance) => distance * self.factor,
            None => 0.0,
        };
        let latency = self.constant_term.as_f32() + distance_factor;
        let latency = Latency::from(latency);
        if latency > self.constraint {
            return Feasibility::Infeasible(latency);
        }
        Feasibility::Feasible(latency)
    }
}

/// Ordered latency is a linear function of the transmission order. The constant term and the
/// factor are mandatory. Latency is small for small order and increases linearly with the order.
/// Factor controls the slope of the linear function.
#[derive(Debug, Clone, Default, Copy)]
pub struct OrderedLatency {
    pub const_param: Latency,
    pub factor: f32,
    pub constraint: Latency,
}

impl Measurable<Latency> for OrderedLatency {
    type P = PayloadInfo;
    type S = LatencyConfig;

    fn with_settings(config: LatencyConfig) -> Self {
        OrderedLatency {
            const_param: config.constant_term.unwrap_or(Latency::from(0.)),
            factor: config.factor.unwrap_or(1.),
            constraint: config.constraint,
        }
    }

    fn measure(&mut self, payload: &PayloadInfo) -> Feasibility<Latency> {
        let order_factor = payload.routing_info.tx_order as f32 * self.factor;
        let latency = self.const_param.as_f32() + order_factor;
        let latency = Latency::from(latency);
        if latency > self.constraint {
            return Feasibility::Infeasible(latency);
        }
        Feasibility::Feasible(latency)
    }
}
