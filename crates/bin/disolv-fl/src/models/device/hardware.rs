use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::message::Metadata;
use disolv_core::metrics::{Bytes, Consumable, Feasibility, Measurable, MetricSettings, Resource};
use disolv_core::metrics::Feasibility::Feasible;
use disolv_models::device::metrics::MegaHertz;
use disolv_models::dist::{DistParams, RngSampler};
use disolv_models::net::metrics::Bandwidth;

use crate::models::device::message::FlPayloadInfo;

#[derive(Debug, Clone, TypedBuilder)]
pub(crate) struct Hardware {
    pub(crate) cpu: CpuUsage,
    pub(crate) storage: Memory,
    pub(crate) bandwidth: BandwidthType,
}

impl Hardware {
    pub fn register_compose(&mut self, metadata: &FlPayloadInfo) {
        self.cpu.consume(metadata);
        self.storage.consume(metadata);
        self.bandwidth.consume(metadata);
    }

    pub fn cpu_available(&self) -> MegaHertz {
        self.cpu.available()
    }

    pub fn memory_available(&self) -> Bytes {
        self.storage.available()
    }

    pub fn bandwidth_available(&self) -> Bandwidth {
        self.bandwidth.available()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CpuSettings {
    pub(crate) cpu_type: String,
    pub(crate) capacity: MegaHertz,
    pub(crate) dist_params: Option<DistParams>,
    pub(crate) constant: Option<MegaHertz>,
    pub(crate) cpu_factor: Option<f64>,
}

impl MetricSettings for CpuSettings {}

#[derive(Debug, Clone)]
pub(crate) enum CpuUsage {
    Random(RandomCpu),
    Constant(ConstantCpu),
    Simple(SimpleCpu),
}

impl Resource<MegaHertz, FlPayloadInfo> for CpuUsage {
    type S = CpuSettings;

    fn with_settings(settings: &CpuSettings) -> Self {
        match settings.cpu_type.to_lowercase().as_str() {
            "random" => CpuUsage::Random(RandomCpu::with_settings(settings)),
            "constant" => CpuUsage::Constant(ConstantCpu::with_settings(settings)),
            "simple" => CpuUsage::Simple(SimpleCpu::with_settings(settings)),
            _ => panic!("Invalid cpu usage model. Supported models: random, constant, simple"),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        match self {
            CpuUsage::Random(random) => random.consume(metadata),
            CpuUsage::Constant(constant) => constant.consume(metadata),
            CpuUsage::Simple(simple) => simple.consume(metadata),
        }
    }

    fn available(&self) -> MegaHertz {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct RandomCpu {
    sampler: RngSampler,
    constant: Option<MegaHertz>,
    usage: MegaHertz,
    capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for RandomCpu {
    type S = CpuSettings;

    fn with_settings(settings: &CpuSettings) -> Self {
        let sampler = match &settings.dist_params {
            Some(dist_params) => RngSampler::new(dist_params.to_owned()),
            None => panic!("Distribution missing for random cpu usage model"),
        };
        Self {
            sampler,
            constant: settings.constant,
            capacity: settings.capacity,
            usage: MegaHertz::default(),
        }
    }

    fn consume(&mut self, _metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        let random_usage = (self.sampler.sample() * self.capacity.as_f64().round()) as u64;
        self.usage = MegaHertz::new(random_usage);
        if let Some(constant) = self.constant {
            self.usage = self.usage + constant;
        }
        Feasible(self.usage)
    }

    fn available(&self) -> MegaHertz {
        self.capacity - self.usage
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ConstantCpu {
    constant_cpu: MegaHertz,
    usage: MegaHertz,
    capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for ConstantCpu {
    type S = CpuSettings;

    fn with_settings(settings: &Self::S) -> Self {
        let constant_cpu = match &settings.constant {
            Some(constant) => constant.clone(),
            None => panic!("Constant missing for constant cpu usage model"),
        };
        Self {
            constant_cpu,
            usage: MegaHertz::default(),
            capacity: settings.capacity,
        }
    }

    fn consume(&mut self, _metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        self.usage = self.constant_cpu;
        Feasible(self.constant_cpu)
    }

    fn available(&self) -> MegaHertz {
        self.capacity - self.usage
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleCpu {
    previous_data: Bytes,
    cpu_factor: f64,
    pub(crate) usage: MegaHertz,
    pub(crate) capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for SimpleCpu {
    type S = CpuSettings;

    fn with_settings(settings: &Self::S) -> Self {
        let cpu_factor: f64 = match settings.cpu_factor {
            Some(factor) => factor,
            None => panic!("Constant missing for constant cpu usage model"),
        };
        Self {
            cpu_factor,
            previous_data: Bytes::default(),
            usage: MegaHertz::default(),
            capacity: settings.capacity,
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        let new_load = (metadata.size() - self.previous_data).as_u64() as f64 * self.cpu_factor;
        if new_load > 0.0 {
            // cpu load should be increased
            self.usage = self.usage + MegaHertz::new(new_load.round() as u64);
        } else {
            self.usage = self.usage - MegaHertz::new(new_load.round() as u64);
        }
        Feasible(self.usage)
    }

    fn available(&self) -> MegaHertz {
        self.capacity - self.usage
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GpuSettings {
    pub(crate) cpu_type: String,
    pub(crate) capacity: MegaHertz,
    pub(crate) dist_params: Option<DistParams>,
    pub(crate) constant: Option<MegaHertz>,
    pub(crate) gpu_factor: Option<f64>,
}

impl MetricSettings for GpuSettings {}

#[derive(Debug, Clone)]
pub(crate) enum Gpu {
    Constant(ConstantGpu),
    Simple(SimpleGpu),
}

impl Resource<MegaHertz, FlPayloadInfo> for Gpu {
    type S = CpuSettings;

    fn with_settings(settings: &CpuSettings) -> Self {
        match settings.cpu_type.to_lowercase().as_str() {
            "constant" => Gpu::Constant(ConstantGpu::with_settings(settings)),
            "simple" => Gpu::Simple(SimpleGpu::with_settings(settings)),
            _ => panic!("Invalid cpu usage model. Supported models: constant, simple"),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        match self {
            Gpu::Constant(constant) => constant.consume(metadata),
            Gpu::Simple(simple) => simple.consume(metadata),
        }
    }

    fn available(&self) -> MegaHertz {
        match self {
            Gpu::Constant(constant) => constant.available(),
            Gpu::Simple(simple) => simple.available(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ConstantGpu {
    constant_cpu: MegaHertz,
    usage: MegaHertz,
    capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for ConstantGpu {
    type S = CpuSettings;

    fn with_settings(settings: &Self::S) -> Self {
        let constant_cpu = match &settings.constant {
            Some(constant) => constant.clone(),
            None => panic!("Constant missing for constant gpu usage model"),
        };
        Self {
            constant_cpu,
            usage: MegaHertz::default(),
            capacity: settings.capacity,
        }
    }

    fn consume(&mut self, _metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        self.usage = self.constant_cpu;
        Feasible(self.constant_cpu)
    }

    fn available(&self) -> MegaHertz {
        self.capacity - self.usage
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleGpu {
    previous_data: Bytes,
    cpu_factor: f64,
    pub(crate) usage: MegaHertz,
    pub(crate) capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for SimpleGpu {
    type S = CpuSettings;

    fn with_settings(settings: &Self::S) -> Self {
        let cpu_factor: f64 = match settings.cpu_factor {
            Some(factor) => factor,
            None => panic!("Constant missing for simple gpu usage model"),
        };
        Self {
            cpu_factor,
            previous_data: Bytes::default(),
            usage: MegaHertz::default(),
            capacity: settings.capacity,
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        let new_load = (metadata.size() - self.previous_data).as_u64() as f64 * self.cpu_factor;
        if new_load > 0.0 {
            // cpu load should be increased
            self.usage = self.usage + MegaHertz::new(new_load.round() as u64);
        } else {
            self.usage = self.usage - MegaHertz::new(new_load.round() as u64);
        }
        Feasible(self.usage)
    }

    fn available(&self) -> MegaHertz {
        self.capacity - self.usage
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemorySettings {
    variant: String,
    starting_memory: Bytes,
    limit: Bytes,
}

impl MetricSettings for MemorySettings {}

#[derive(Debug, Clone)]
pub enum Memory {
    Simple(SimpleMemory),
}

impl Resource<Bytes, FlPayloadInfo> for Memory {
    type S = MemorySettings;

    fn with_settings(settings: &Self::S) -> Self {
        todo!()
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bytes> {
        todo!()
    }

    fn available(&self) -> Bytes {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SimpleMemory {
    available: Bytes,
    consumed: Bytes,
}

impl Consumable<Bytes, FlPayloadInfo> for SimpleMemory {
    type S = MemorySettings;

    fn with_settings(settings: Self::S) -> Self {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bytes> {
        todo!()
    }

    fn available(&self) -> Bytes {
        todo!()
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BandwidthConfig {
    pub variant: String,
}

impl MetricSettings for BandwidthConfig {}

#[derive(Debug, Clone)]
pub enum BandwidthType {
    Simple(SimpleBandwidth),
}

impl Consumable<Bandwidth, FlPayloadInfo> for BandwidthType {
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "simple" => BandwidthType::Simple(SimpleBandwidth::with_settings(settings)),
            _ => panic!("Unsupported bandwidth variant {}.", settings.variant),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Simple(constant) => constant.reset(),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bandwidth> {
        match self {
            Self::Simple(constant) => constant.consume(metadata),
        }
    }

    fn available(&self) -> Bandwidth {
        match self {
            Self::Simple(constant) => constant.available(),
        }
    }
}

/// A SimpleBandwidth is a bandwidth that is constant for all time steps.
#[derive(Debug, Clone, Default)]
pub struct SimpleBandwidth {
    pub bandwidth: Bandwidth,
}

impl Consumable<Bandwidth, FlPayloadInfo> for SimpleBandwidth {
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            ..Default::default()
        }
    }

    fn reset(&mut self) {
        self.bandwidth = Bandwidth::default();
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bandwidth> {
        self.bandwidth = self.bandwidth + Bandwidth::new(metadata.size().as_u64());
        Feasibility::Feasible(self.bandwidth)
    }

    fn available(&self) -> Bandwidth {
        self.bandwidth
    }
}
