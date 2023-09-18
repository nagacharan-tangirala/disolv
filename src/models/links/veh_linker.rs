use crate::reader::activation::DeviceId;
use crate::sim::vanet::Link;
use crate::utils::config::VehicleLinker;

#[derive(Clone, Debug, Copy)]
pub(crate) enum VehLinkerType {
    Simple(SimpleVehLinker),
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct SimpleVehLinker {
    pub(crate) mesh_range: f32,
    pub(crate) bs_range: f32,
    pub(crate) rsu_range: f32,
}

impl SimpleVehLinker {
    pub(crate) fn new(linker_settings: VehicleLinker) -> Self {
        Self {
            mesh_range: linker_settings.mesh_range,
            bs_range: linker_settings.bs_range,
            rsu_range: linker_settings.rsu_range,
        }
    }

    pub(crate) fn find_vehicle_mesh_links(
        &self,
        v2v_links_opt: Option<Link>,
    ) -> Option<Vec<DeviceId>> {
        match v2v_links_opt {
            Some(v2v_links) => {
                let mut selected_vehicle_ids = Vec::with_capacity(v2v_links.0.len());
                let (veh_ids, distances) = v2v_links;
                for (veh_id, distance) in veh_ids.iter().zip(distances.iter()) {
                    if *distance <= self.mesh_range {
                        selected_vehicle_ids.push(*veh_id);
                    }
                }
                Some(selected_vehicle_ids)
            }
            None => None,
        }
    }

    pub(crate) fn find_rsu_link(&self, v2rsu_links_opt: Option<Link>) -> Option<DeviceId> {
        match v2rsu_links_opt {
            Some(v2rsu_links) => {
                let mut selected_rsu_id: DeviceId = 0;
                let mut rsu_distance: f32 = 0.0;
                let (rsu_ids, distances) = v2rsu_links;
                for (rsu_id, distance) in rsu_ids.iter().zip(distances.iter()) {
                    if *distance <= self.rsu_range {
                        selected_rsu_id = *rsu_id;
                        rsu_distance = *distance;
                    }
                }
                Some(selected_rsu_id)
            }
            None => None,
        }
    }

    pub(crate) fn find_bs_link(&self, v2bs_links_opt: Option<Link>) -> Option<DeviceId> {
        match v2bs_links_opt {
            Some(v2bs_links) => {
                let mut selected_bs_id: DeviceId = 0;
                let mut bs_distance: f32 = 0.0;
                let (bs_ids, distances) = v2bs_links;
                for (bs_id, distance) in bs_ids.iter().zip(distances.iter()) {
                    if *distance <= self.bs_range {
                        selected_bs_id = *bs_id;
                        bs_distance = *distance;
                    }
                }
                Some(selected_bs_id)
            }
            None => None,
        }
    }
}
