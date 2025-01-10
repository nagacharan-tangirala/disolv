use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::message::Metadata;
use disolv_core::metrics::{Bytes, Consumable, Feasibility, MetricSettings, Resource};
use disolv_core::metrics::Feasibility::{Feasible, Infeasible};
use disolv_core::model::{Model, ModelSettings};
use disolv_models::device::metrics::MegaHertz;
use disolv_models::dist::{DistParams, UnitSampler};
use disolv_models::net::metrics::Bandwidth;

use crate::models::device::message::FlPayloadInfo;

#[derive(Deserialize, Debug, Clone)]
pub struct HardwareSettings {
    pub cpu: CpuSettings,
    pub gpu: GpuSettings,
    pub memory: MemorySettings,
    pub bandwidth: BandwidthSettings,
}

impl ModelSettings for HardwareSettings {}

#[derive(Debug, Clone, TypedBuilder)]
pub(crate) struct Hardware {
    pub(crate) cpu: Cpu,
    pub(crate) gpu: Gpu,
    pub(crate) storage: Memory,
    pub(crate) bandwidth: BandwidthType,
}

impl Model for Hardware {
    type Settings = HardwareSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        Self {
            cpu: Cpu::with_settings(&settings.cpu),
            gpu: Gpu::with_settings(&settings.gpu),
            storage: Memory::with_settings(&settings.memory),
            bandwidth: BandwidthType::with_settings(settings.bandwidth.clone()),
        }
    }
}

impl Hardware {
    pub fn register_compose(&mut self, metadata: &FlPayloadInfo) {
        self.cpu.consume(metadata);
        self.storage.consume(metadata);
        self.bandwidth.consume(metadata);
        self.gpu.consume(metadata);
    }

    pub fn register_training(&mut self, data_size: usize) {
        self.gpu.register_training(data_size);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpuSettings {
    pub(crate) cpu_type: String,
    pub(crate) capacity: MegaHertz,
    pub(crate) dist_params: Option<DistParams>,
    pub(crate) constant: Option<MegaHertz>,
    pub(crate) cpu_factor: Option<f64>,
}

impl MetricSettings for CpuSettings {}

#[derive(Debug, Clone)]
pub(crate) enum Cpu {
    Random(RandomCpu),
    Constant(ConstantCpu),
    Simple(SimpleCpu),
}

impl Resource<MegaHertz, FlPayloadInfo> for Cpu {
    type S = CpuSettings;

    fn with_settings(settings: &CpuSettings) -> Self {
        match settings.cpu_type.to_lowercase().as_str() {
            "random" => Cpu::Random(RandomCpu::with_settings(settings)),
            "constant" => Cpu::Constant(ConstantCpu::with_settings(settings)),
            "simple" => Cpu::Simple(SimpleCpu::with_settings(settings)),
            _ => panic!("Invalid cpu usage model. Supported models: random, constant, simple"),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        match self {
            Cpu::Random(random) => random.consume(metadata),
            Cpu::Constant(constant) => constant.consume(metadata),
            Cpu::Simple(simple) => simple.consume(metadata),
        }
    }

    fn available(&self) -> MegaHertz {
        match self {
            Cpu::Random(random) => random.available(),
            Cpu::Constant(constant) => constant.available(),
            Cpu::Simple(simple) => simple.available(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RandomCpu {
    sampler: UnitSampler,
    constant: Option<MegaHertz>,
    usage: MegaHertz,
    capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for RandomCpu {
    type S = CpuSettings;

    fn with_settings(settings: &CpuSettings) -> Self {
        let sampler = match &settings.dist_params {
            Some(dist_params) => UnitSampler::new(dist_params.to_owned()),
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
            None => panic!("Constant missing for simple cpu usage model"),
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
pub struct GpuSettings {
    pub(crate) gpu_type: String,
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
    type S = GpuSettings;

    fn with_settings(settings: &GpuSettings) -> Self {
        match settings.gpu_type.to_lowercase().as_str() {
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

impl Gpu {
    fn register_training(&mut self, data_size: usize) {
        match self {
            Gpu::Simple(simple) => simple.register_training(data_size),
            _ => panic!("unsupported"),
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
    type S = GpuSettings;

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
    gpu_factor: f64,
    pub(crate) usage: MegaHertz,
    pub(crate) capacity: MegaHertz,
}

impl Resource<MegaHertz, FlPayloadInfo> for SimpleGpu {
    type S = GpuSettings;

    fn with_settings(settings: &Self::S) -> Self {
        let gpu_factor: f64 = match settings.gpu_factor {
            Some(factor) => factor,
            None => panic!("Constant missing for simple gpu usage model"),
        };
        Self {
            gpu_factor,
            previous_data: Bytes::default(),
            usage: MegaHertz::default(),
            capacity: settings.capacity,
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<MegaHertz> {
        let new_load = (metadata.size() - self.previous_data).as_u64() as f64 * self.gpu_factor;
        if new_load > 0.0 {
            // gpu load should be increased
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

impl SimpleGpu {
    fn register_training(&mut self, data_size: usize) {
        self.usage = MegaHertz::new((data_size as f64 * self.gpu_factor).round() as u64);
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
        match settings.variant.to_lowercase().as_str() {
            "simple" => Memory::Simple(SimpleMemory::with_settings(settings)),
            _ => panic!("Invalid memory model. Only simple is supported."),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bytes> {
        match self {
            Memory::Simple(memory) => memory.consume(metadata),
        }
    }

    fn available(&self) -> Bytes {
        match self {
            Memory::Simple(memory) => memory.available(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleMemory {
    available: Bytes,
    consumed: Bytes,
    limit: Bytes,
}

impl Resource<Bytes, FlPayloadInfo> for SimpleMemory {
    type S = MemorySettings;

    fn with_settings(settings: &Self::S) -> Self {
        Self {
            available: settings.starting_memory,
            limit: settings.limit,
            consumed: Bytes::default(),
        }
    }

    fn consume(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Bytes> {
        let new_consumed = self.consumed + metadata.size();
        if new_consumed < self.limit {
            self.consumed = new_consumed;
            return Feasible(self.consumed);
        }
        Infeasible(new_consumed)
    }

    fn available(&self) -> Bytes {
        self.available
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BandwidthSettings {
    pub variant: String,
}

impl MetricSettings for BandwidthSettings {}

#[derive(Debug, Clone)]
pub enum BandwidthType {
    Simple(SimpleBandwidth),
}

impl Consumable<Bandwidth, FlPayloadInfo> for BandwidthType {
    type S = BandwidthSettings;

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
    type S = BandwidthSettings;

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
        Feasible(self.bandwidth)
    }

    fn available(&self) -> Bandwidth {
        self.bandwidth
    }
}
