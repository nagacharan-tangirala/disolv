use pavenet_config::config::base::{LinkConfig, LinkerSettings};

pub const LINKER_SIZE: usize = 5;

#[derive(Clone, Debug, Copy)]
pub(crate) enum LinkerType {
    Simple(BasicLinker),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct BasicLinker {
    pub(crate) link_config: [Option<LinkConfig>; LINKER_SIZE],
}

impl BasicLinker {
    pub(crate) fn new(linker_settings: LinkerSettings) -> Self {
        let mut link_config = [None; LINKER_SIZE];
        for idx in 0..linker_settings.len() {
            link_config[idx] = Some(linker_settings[idx]);
        }
        Self { link_config }
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
                    if *distance <= self.settings_handler.linker_in_use.mesh_range {
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
                    if *distance <= self.settings_handler.linker_in_use.rsu_range {
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
                    if *distance <= self.settings_handler.linker_in_use.bs_range {
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
