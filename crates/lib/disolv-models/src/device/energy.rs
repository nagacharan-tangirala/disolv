use serde::Deserialize;

use disolv_core::metrics::Feasibility;
use disolv_core::metrics::Measurable;
use disolv_core::metrics::MetricSettings;

use crate::net::message::PayloadInfo;
use crate::net::message::TxMetrics;

use super::metrics::Energy;

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

impl Measurable<Energy> for EnergyType {
    type P = PayloadInfo;
    type S = EnergySettings;
    type T = TxMetrics;

    fn with_settings(settings: &Self::S) -> Self {
        match settings.name.to_lowercase().as_str() {
            "proportional" => EnergyType::Proportional(ProportionalEnergy::with_settings(settings)),
            _ => panic!("Unsupported energy variant {}", settings.name),
        }
    }

    fn measure(&mut self, tx_report: &TxMetrics, metadata: &PayloadInfo) -> Feasibility<Energy> {
        match self {
            EnergyType::Proportional(energy) => energy.measure(tx_report, metadata),
            EnergyType::TrainingLength(energy) => energy.measure(tx_report, metadata),
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

    fn with_settings(settings: &Self::S) -> Self {
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

#[derive(Copy, Clone, Debug)]
pub struct TrainingLengthEnergy {
    pub factor: u64,
}

impl Measurable<Energy> for TrainingLengthEnergy {
    type P = PayloadInfo;
    type S = EnergySettings;
    type T = TxMetrics;

    fn with_settings(settings: &Self::S) -> Self {
        Self {
            factor: settings.factor,
        }
    }

    fn measure(&mut self, tx_report: &TxMetrics, metadata: &PayloadInfo) -> Feasibility<Energy> {
        let energy = Energy::new(tx_report.payload_size.as_u64() * self.factor);
        Feasibility::Feasible(energy)
    }
}
