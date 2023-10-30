use crate::bucket::TimeStamp;
use crate::payload::{GPayload, PayloadContent, PayloadMetadata};
use crate::response::Queryable;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric:
    Default + AddAssign + Add<Output = Self> + PartialEq + PartialOrd + Copy + Clone + Send + Sync
{
    fn as_f32(&self) -> f32;
}

/// A trait that can be used to contain the measurement process of a metric. It must be applied on
/// each individual variant of the metric.
pub trait Measurable<M, P>: Clone + Send + Sync
where
    M: Metric,
    P: PayloadMetadata,
{
    fn measure(&mut self, metadata: &P) -> M;
}

/// A trait that contains possible parameters with which a metric variant can be configured. This
/// can be used to mark the struct that contains the configuration parameters applicable to all
/// the variants. Use this to mark a struct that can be read from a configuration file.
pub trait VariantConfig<M>: Clone + Send + Sync
where
    M: Metric,
{
}

/// A trait that represents a variant of a metric. This is used to implement the variations of the
/// metric that may involve different methods of measurement. For example, a metric can be latency
/// and the variants can be the different methods of measuring the latency (constant, linear, etc.).
/// It is recommended to use an enum to implement this trait. The measure method can be used inside
/// a match statement to call the appropriate method of measurement.
pub trait MetricVariant<C, M, P>: Clone + Send + Sync
where
    C: VariantConfig<M>,
    M: Metric,
    P: PayloadMetadata,
{
    fn new(variant_config: C) -> Self;
    fn measure(&mut self, metadata: &P) -> M;
}

/// An enum that represents the feasibility of a metric. This is used as return type of the
/// feasibility evaluation so that the caller can get the feasibility and the actual value of the
/// metric that was measured.
#[derive(Clone, Copy, Debug)]
pub enum Feasibility<M>
where
    M: Metric,
{
    Feasible(M),
    Infeasible(M),
}

/// A generic struct that can be used to contain a metric data related to a radio that can be
/// measured. A user can define a constraint to evaluate the feasibility of the metric. This can
/// be used for metrics that are not cumulative in nature. For example, latency. The latency is
/// different for different payloads. Hence, the latency value is not required to be cumulative.
#[derive(Default, Clone, Copy, Debug)]
pub struct GRadioMeasurement<C, M, P, V>
where
    C: VariantConfig<M>,
    M: Metric,
    P: PayloadMetadata,
    V: MetricVariant<C, M, P>,
{
    constraint: Option<M>,
    variant: V,
    _phantom: std::marker::PhantomData<fn() -> (C, P)>,
}

impl<C, M, P, V> GRadioMeasurement<C, M, P, V>
where
    C: VariantConfig<M>,
    M: Metric,
    P: PayloadMetadata,
    V: MetricVariant<C, M, P>,
{
    pub fn new(variant_config: C, constraint: Option<M>) -> Self {
        let variant = V::new(variant_config);
        Self {
            constraint,
            variant,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn check_feasibility(&mut self, metadata: &P) -> Feasibility<M> {
        let measured = self.variant.measure(metadata);
        return match self.constraint {
            Some(constraint) => {
                if constraint >= measured {
                    return Feasibility::Feasible(measured);
                }
                Feasibility::Infeasible(measured)
            }
            None => Feasibility::Feasible(measured),
        };
    }
}

/// A generic struct containing the resource availability and the consumed resource.
#[derive(Default, Clone, Copy, Debug)]
pub struct GRadioResource<C, M, P, V>
where
    C: VariantConfig<M>,
    M: Metric,
    P: PayloadMetadata,
    V: MetricVariant<C, M, P>,
{
    available: M,
    variant: V,
    pub used: M,
    _phantom: std::marker::PhantomData<fn() -> (C, P)>,
}

impl<C, M, P, V> GRadioResource<C, M, P, V>
where
    C: VariantConfig<M>,
    M: Metric,
    P: PayloadMetadata,
    V: MetricVariant<C, M, P>,
{
    pub fn new(variant_config: C, available: M) -> Self {
        let variant = V::new(variant_config);
        Self {
            available,
            variant,
            used: M::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.used = M::default();
    }

    pub fn availability(&self) -> f32 {
        self.used.as_f32() / self.available.as_f32()
    }

    pub fn add_used(&mut self, used: M) {
        self.used += used;
    }

    pub fn check_feasibility(&mut self, metadata: &P) -> Feasibility<M> {
        let measured = self.variant.measure(metadata);
        let updated_used = self.used + measured;
        return if self.available >= updated_used {
            self.add_used(measured);
            Feasibility::Feasible(measured)
        } else {
            Feasibility::Infeasible(measured)
        };
    }
}

/// A trait that represents a radio that can be used to transfer data. It performs the actual
/// data transfer and can be used to measure the radio usage.
pub trait Channel<C, P, Q, T>
where
    C: PayloadContent<Q>,
    P: PayloadMetadata,
    Q: Queryable,
    T: TimeStamp,
{
    fn can_transfer(&mut self, payloads: Vec<GPayload<C, P, Q>>) -> Vec<GPayload<C, P, Q>>;
    fn can_forward(&mut self, payloads: Vec<GPayload<C, P, Q>>) -> Vec<GPayload<C, P, Q>>;
    fn consume(&mut self, payload: &GPayload<C, P, Q>);
}

/// A trait to represent a type that holds statistics of the radio usage for incoming data.
pub trait IncomingStats<M>: Clone + Copy + Debug
where
    M: PayloadMetadata,
{
    fn update(&mut self, metadata: &Vec<M>);
}

/// A trait to represent a type that holds statistics of the radio usage for outgoing data.
pub trait OutgoingStats<M>: Clone + Copy + Debug
where
    M: PayloadMetadata,
{
    fn update(&mut self, metadata: &M);
}