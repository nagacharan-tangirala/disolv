use crate::bucket::TimeStamp;
use crate::payload::{Payload, PayloadContent, PayloadMetadata};
use crate::response::Queryable;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

/// A trait to represent the name of the metric that can quantify the radio usage. This is used to
/// identify the various types of metrics that can be used. It is recommended to use an enum to
/// implement this trait.
pub trait MetricName: Default + Copy + Clone + Send + Sync {}

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric:
    Default + AddAssign + Add<Output = Self> + PartialEq + PartialOrd + Copy + Clone + Send + Sync
{
    fn as_f32(&self) -> f32;
}

/// A trait that can be used to contain the measurement process of a metric. It is applied on the
/// variant of the metric so that different variants can have different methods of measurement.
pub trait Measurable<M, P>
where
    M: Metric,
    P: PayloadMetadata,
{
    fn measure(&mut self, metadata: &Vec<P>) -> M;
}

/// A trait that represents a variant of a metric. This is used to implement the variations of the
/// metric that may involve different methods of measurement. For example, a metric can be latency
/// and the variants can be the different methods of measuring the latency (constant, linear, etc.).
pub trait MetricVariant<M, P>: Measurable<M, P> + Default + Copy + Clone + Send + Sync
where
    M: Metric,
    P: PayloadMetadata,
{
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
/// measured. A user can define a constraint to evaluate the feasibility of the metric.
#[derive(Default, Clone, Copy, Debug)]
pub struct RadioMeasurement<M, N, P, V>
where
    M: Metric,
    N: MetricName,
    P: PayloadMetadata,
    V: MetricVariant<M, P>,
{
    pub name: N,
    constraint: Option<M>,
    variant: V,
    _phantom: std::marker::PhantomData<fn() -> P>,
}

impl<M, N, P, V> RadioMeasurement<M, N, P, V>
where
    M: Metric,
    N: MetricName,
    P: PayloadMetadata,
    V: MetricVariant<M, P>,
{
    pub fn new(metric_type: N, variant: V, constraint: Option<M>) -> Self {
        Self {
            name: metric_type,
            constraint,
            variant,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn check_feasibility(&mut self, metadata: &Vec<P>) -> Feasibility<M> {
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
pub struct RadioResource<M, N, P, V>
where
    M: Metric,
    N: MetricName,
    P: PayloadMetadata,
    V: MetricVariant<M, P>,
{
    pub name: N,
    available: M,
    variant: V,
    pub used: M,
    _phantom: std::marker::PhantomData<fn() -> P>,
}

impl<M, N, P, V> RadioResource<M, N, P, V>
where
    M: Metric,
    N: MetricName,
    P: PayloadMetadata,
    V: MetricVariant<M, P>,
{
    pub fn new(metric_type: N, available: M, variant: V) -> Self {
        Self {
            name: metric_type,
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

    pub fn check_feasibility(&mut self, metadata: &Vec<P>) -> Feasibility<M> {
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
pub trait Radio<C, M, N, P, Q, T>
where
    C: PayloadContent<Q>,
    M: Metric,
    N: MetricName,
    P: PayloadMetadata,
    Q: Queryable,
    T: TimeStamp,
{
    fn can_transfer(&mut self, payloads: Vec<Payload<C, P, Q>>) -> Vec<Payload<C, P, Q>>;
    fn can_forward(&mut self, payloads: Vec<Payload<C, P, Q>>) -> Vec<Payload<C, P, Q>>;
    fn consume(&mut self, payload: &Payload<C, P, Q>);
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
