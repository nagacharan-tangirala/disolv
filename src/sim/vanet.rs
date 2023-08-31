use krabmaga::hashbrown::HashMap;

type Link = (Vec<f32>, Vec<f32>);

pub struct Vanet {
    pub mesh_links: MeshLinks,
    pub infra_links: InfraLinks,
}

pub struct MeshLinks {
    pub v2v_links: HashMap<i64, Link>,
    pub rsu2rsu_links: HashMap<i64, Link>,
    pub v2rsu_links: HashMap<i64, Link>,
}

pub struct InfraLinks {
    pub v2bs_links: HashMap<i64, Link>,
    pub rsu2bs_links: HashMap<i64, Link>,
    pub bs2c_links: HashMap<i64, Link>,
}

impl MeshLinks {
    pub(crate) fn new() -> Self {
        Self {
            v2v_links: HashMap::new(),
            rsu2rsu_links: HashMap::new(),
            v2rsu_links: HashMap::new(),
        }
    }
}

impl InfraLinks {
    pub(crate) fn new() -> Self {
        Self {
            v2bs_links: HashMap::new(),
            rsu2bs_links: HashMap::new(),
            bs2c_links: HashMap::new(),
        }
    }
}

impl Vanet {
    pub(crate) fn new(mesh_links: MeshLinks, infra_links: InfraLinks) -> Self {
        Self {
            mesh_links,
            infra_links,
        }
    }

    // fn read_v2v_links(&self) -> HashMap<i32, Vec<i32>> {
    //     let v2v_link_file = Path::new(&self.config_path).join(&self.config.link_files.v2v_links);
    //     if v2v_link_file.exists() == false {
    //         panic!("V2V link file is not found.");
    //     }
    //     let v2v_links: HashMap<i32, Vec<i32>> = match self
    //         .links_reader
    //         .read_dynamic_links(v2v_link_file, VEHICLES_STR)
    //     {
    //         Ok(v2v_links) => v2v_links,
    //         Err(e) => {
    //             panic!("Error while reading V2V links: {}", e);
    //         }
    //     };
    //     return v2v_links;
    // }
}
