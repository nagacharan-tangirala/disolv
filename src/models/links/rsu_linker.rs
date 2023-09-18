use crate::reader::activation::DeviceId;
use crate::sim::vanet::Link;
use crate::utils::config::RSULinker;

#[derive(Clone, Debug, Copy)]
pub(crate) enum RSULinkerType {
    Simple(SimpleRSULinker),
}

#[derive(Clone, Debug, Copy, Default)]
pub(crate) struct SimpleRSULinker {
    pub(crate) mesh_range: f32,
    pub(crate) bs_range: f32,
}

impl SimpleRSULinker {
    pub(crate) fn new(linker_settings: RSULinker) -> Self {
        Self {
            mesh_range: linker_settings.mesh_range,
            bs_range: linker_settings.bs_range,
        }
    }

    pub(crate) fn find_rsu_mesh_links(
        &self,
        rsu2rsu_links_opt: Option<Link>,
    ) -> Option<Vec<DeviceId>> {
        match rsu2rsu_links_opt {
            Some(rsu_links) => {
                let mut selected_rsu_ids = Vec::with_capacity(rsu_links.0.len());
                let (rsu_ids, distances) = rsu_links;
                for (rsu_id, distance) in rsu_ids.iter().zip(distances.iter()) {
                    if *distance <= self.mesh_range {
                        selected_rsu_ids.push(*rsu_id);
                    }
                }
                Some(selected_rsu_ids)
            }
            None => None,
        }
    }

    pub(crate) fn find_bs_link(&self, rsu2bs_links_opt: Option<Link>) -> Option<DeviceId> {
        match rsu2bs_links_opt {
            Some(rsu2bs_links) => {
                let mut selected_bs_id: DeviceId = 0;
                let mut bs_distance: f32 = 0.0;
                let (bs_ids, distances) = rsu2bs_links;
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
