use crate::device::vehicle::VehiclePayload;
use crate::utils::ds_config::DataSourceSettings;

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicSimplifier;

#[derive(Clone, Debug, Copy)]
pub(crate) struct RandomSimplifier;

#[derive(Clone, Debug, Copy)]
pub(crate) enum SimplifierType {
    Basic(BasicSimplifier),
    Random(RandomSimplifier),
}

trait VehicleSimplifier {
    fn simplify_payload(&self, vehicle_payload: VehiclePayload) -> VehiclePayload;
}

trait RSUSimplifier {
    fn simplify_payload(&self, vehicle_payload: VehiclePayload) -> VehiclePayload;
}
