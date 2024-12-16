use serde::Deserialize;

use disolv_core::message::Metadata;
use disolv_core::metrics::Feasibility;
use disolv_core::metrics::Measurable;
use disolv_core::metrics::MetricSettings;
use disolv_models::device::metrics::Energy;

use crate::models::device::message::FlPayloadInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct EnergySettings {
    pub name: String,
    pub factor: u64,
    pub static_power: Option<Energy>,
}

impl MetricSettings for EnergySettings {}

#[derive(Copy, Clone, Debug)]
pub enum EnergyType {
    Proportional(ProportionalEnergy),
    TrainingLength(TrainingLengthEnergy),
}

impl Measurable<Energy, FlPayloadInfo> for EnergyType {
    type S = EnergySettings;

    fn with_settings(settings: &Self::S) -> Self {
        match settings.name.to_lowercase().as_str() {
            "proportional" => EnergyType::Proportional(ProportionalEnergy::with_settings(settings)),
            _ => panic!("Unsupported energy variant {}", settings.name),
        }
    }

    fn measure(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Energy> {
        match self {
            EnergyType::Proportional(energy) => energy.measure(metadata),
            EnergyType::TrainingLength(energy) => energy.measure(metadata),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ProportionalEnergy {
    pub factor: u64,
    pub static_energy: Energy,
}

impl Measurable<Energy, FlPayloadInfo> for ProportionalEnergy {
    type S = EnergySettings;

    fn with_settings(settings: &Self::S) -> Self {
        Self {
            factor: settings.factor,
            static_energy: settings.static_power.expect("static component not given"),
        }
    }

    fn measure(&mut self, metadata: &FlPayloadInfo) -> Feasibility<Energy> {
        let energy = self.static_energy + Energy::new(metadata.size().as_u64() * self.factor);
        Feasibility::Feasible(self.static_energy)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TrainingLengthEnergy {
    pub factor: u64,
}

impl<P: Metadata> Measurable<Energy, P> for TrainingLengthEnergy {
    type S = EnergySettings;

    fn with_settings(settings: &Self::S) -> Self {
        Self {
            factor: settings.factor,
        }
    }

    fn measure(&mut self, metadata: &P) -> Feasibility<Energy> {
        let energy = Energy::new(metadata.size().as_u64() * self.factor);
        Feasibility::Feasible(energy)
    }
}
