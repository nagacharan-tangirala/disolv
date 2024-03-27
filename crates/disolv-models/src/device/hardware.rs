use crate::device::metrics::Energy;
use crate::device::metrics::MegaHertz;
use crate::net::message::{PayloadInfo, TxMetrics};
use crate::net::metrics::Bytes;
use disolv_core::metrics::Consumable;
use disolv_core::metrics::{Feasibility, Measurable, MetricSettings, NonReplenishable};
use log::error;
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone)]
pub struct EnergySettings {
    pub name: String,
    pub factor: u64,
    pub static_power: Option<Energy>,
}

impl MetricSettings for EnergySettings {}

#[derive(Copy, Clone, Debug)]
pub enum EnergyType {
    Proportional(ProportionalEnergy),
}

impl Measurable<Energy> for EnergyType {
    type P = PayloadInfo;
    type S = EnergySettings;
    type T = TxMetrics;

    fn with_settings(settings: Self::S) -> Self {
        match settings.name.to_lowercase().as_str() {
            "proportional" => EnergyType::Proportional(ProportionalEnergy::with_settings(settings)),
            _ => panic!("Unsupported energy variant {}", settings.name),
        }
    }

    fn measure(&mut self, tx_report: &TxMetrics, metadata: &PayloadInfo) -> Feasibility<Energy> {
        match self {
            EnergyType::Proportional(energy) => energy.measure(tx_report, metadata),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ProportionalEnergy {
    pub factor: u64,
    pub static_energy: Energy,
}

impl Measurable<Energy> for ProportionalEnergy {
    type P = PayloadInfo;
    type S = EnergySettings;
    type T = TxMetrics;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            factor: settings.factor,
            static_energy: settings.static_power.expect("static component not given"),
        }
    }

    fn measure(&mut self, tx_report: &TxMetrics, metadata: &PayloadInfo) -> Feasibility<Energy> {
        let energy =
            self.static_energy + Energy::new(tx_report.payload_size.as_u64() * self.factor);
        Feasibility::Feasible(self.static_energy)
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct StorageSettings {
    pub variant: String,
    pub limit: Bytes,
}

impl MetricSettings for StorageSettings {}

#[derive(Debug, Copy, Clone)]
pub enum StorageType {
    Constant(ConstantStorage),
}

impl NonReplenishable<Bytes> for StorageType {
    type P = PayloadInfo;
    type S = StorageSettings;

    fn with_settings(settings: Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "constant" => StorageType::Constant(ConstantStorage::with_settings(settings)),
            _ => panic!("Unsupported storage type {}", settings.variant),
        }
    }

    fn consume(&mut self, metadata: &Self::P) {
        match self {
            StorageType::Constant(storage) => storage.consume(metadata),
        }
    }

    fn reserve(&mut self, metadata: &Self::P) -> Feasibility<Bytes> {
        match self {
            StorageType::Constant(storage) => storage.reserve(metadata),
        }
    }

    fn available(&self) -> Bytes {
        match self {
            StorageType::Constant(storage) => storage.available(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ConstantStorage {
    pub available: Bytes,
    pub reserved: Bytes,
    limit: Bytes,
}

impl NonReplenishable<Bytes> for ConstantStorage {
    type P = PayloadInfo;
    type S = StorageSettings;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            limit: settings.limit,
            ..Default::default()
        }
    }

    fn consume(&mut self, metadata: &Self::P) {
        self.available = self.reserved;
    }

    fn reserve(&mut self, metadata: &Self::P) -> Feasibility<Bytes> {
        let to_reserve = self.available - metadata.total_size;
        if self.reserved + to_reserve > self.limit {
            Feasibility::Infeasible(self.available)
        } else {
            self.reserved += to_reserve;
            Feasibility::Feasible(self.available)
        }
    }

    fn available(&self) -> Bytes {
        self.available
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct CpuSettings {
    pub variant: String,
    pub limit: MegaHertz,
}

impl MetricSettings for CpuSettings {}

#[derive(Copy, Clone, Debug)]
pub enum CpuType {
    Constant(ConstantCpu),
}

impl Consumable<MegaHertz> for CpuType {
    type P = PayloadInfo;
    type S = CpuSettings;

    fn with_settings(settings: Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "constant" => CpuType::Constant(ConstantCpu::with_settings(settings)),
            _ => {
                error!("Only Constant bandwidth variant is supported.");
                panic!("Unsupported bandwidth variant {}.", settings.variant);
            }
        }
    }

    fn reset(&mut self) {
        match self {
            CpuType::Constant(cpu) => cpu.reset(),
        }
    }

    fn consume(&mut self, metadata: &Self::P) {
        match self {
            CpuType::Constant(cpu) => cpu.consume(metadata),
        }
    }

    fn constraint(&self) -> MegaHertz {
        match self {
            CpuType::Constant(cpu) => cpu.constraint(),
        }
    }

    fn available(&self) -> MegaHertz {
        match self {
            CpuType::Constant(cpu) => cpu.available(),
        }
    }

    fn reserve(&mut self, metadata: &Self::P) -> Feasibility<MegaHertz> {
        match self {
            CpuType::Constant(cpu) => cpu.reserve(metadata),
        }
    }
}
#[derive(Default, Copy, Clone, Debug)]
pub struct ConstantCpu {
    pub available: MegaHertz,
    pub reserved: MegaHertz,
    pub limit: MegaHertz,
}

impl Consumable<MegaHertz> for ConstantCpu {
    type P = PayloadInfo;
    type S = CpuSettings;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            limit: settings.limit,
            ..Default::default()
        }
    }

    fn reset(&mut self) {
        self.available = self.limit;
        self.reserved = MegaHertz::default();
    }

    fn consume(&mut self, metadata: &Self::P) {
        self.available = self.reserved;
    }

    fn constraint(&self) -> MegaHertz {
        self.limit
    }

    fn available(&self) -> MegaHertz {
        self.available
    }

    fn reserve(&mut self, metadata: &Self::P) -> Feasibility<MegaHertz> {
        let mut cpu_load = MegaHertz::default();
        metadata
            .data_blobs
            .iter()
            .for_each(|d| cpu_load += d.cpu_load);

        if self.reserved + cpu_load > self.limit {
            Feasibility::Infeasible(self.available)
        } else {
            self.reserved += cpu_load;
            Feasibility::Feasible(self.available)
        }
    }
}

#[derive(Debug, Copy, Clone, TypedBuilder)]
pub struct Hardware {
    pub storage: StorageType,
    pub energy: EnergyType,
    pub cpu: CpuType,
}
