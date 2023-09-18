use crate::models::composer::UplinkPayload;
use crate::utils::config::SimplifierSettings;

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicSimplifier {
    pub(crate) compression_factor: f32,
    pub(crate) sampling_factor: f32,
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct RandomSimplifier;

#[derive(Clone, Debug, Copy)]
pub(crate) enum SimplifierType {
    Basic(BasicSimplifier),
    Random(RandomSimplifier),
}

impl BasicSimplifier {
    pub(crate) fn new(simplifier_settings: SimplifierSettings) -> Self {
        Self {
            compression_factor: simplifier_settings.compression_factor,
            sampling_factor: simplifier_settings.sampling_factor,
        }
    }
    pub(crate) fn simplify_payload(&self, payload: UplinkPayload) -> UplinkPayload {
        let mut simplified_payload = UplinkPayload::default();
        return simplified_payload;
    }
}

impl RandomSimplifier {
    pub(crate) fn new(simplifier_settings: SimplifierSettings) -> Self {
        Self {}
    }

    pub(crate) fn simplify_payload(&self, payload: UplinkPayload) -> UplinkPayload {
        return payload;
    }
}
