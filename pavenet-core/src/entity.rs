use crate::entity::class::NodeClass;
use crate::entity::kind::NodeType;
use pavenet_engine::entity::NodeId;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: NodeClass,
}

pub mod class {
    use pavenet_engine::entity::Tier;
    use serde::Deserialize;
    use std::fmt::{Debug, Display};
    use std::hash::Hash;

    #[derive(Deserialize, Default, Clone, Eq, Copy, PartialOrd, Ord)]
    #[serde(tag = "class_name", content = "class_order")]
    pub enum NodeClass {
        #[default]
        None,
        Vehicle5G(i32),
        RSU5G(i32),
        BaseStation5G(i32),
        Controller(i32),
    }

    impl Hash for NodeClass {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.as_u32().hash(state);
        }
    }

    impl Debug for NodeClass {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NodeClass::None => write!(f, "None"),
                NodeClass::Vehicle5G(x) => write!(f, "Vehicle 5G({})", x),
                NodeClass::RSU5G(x) => write!(f, "RSU 5G({})", x),
                NodeClass::BaseStation5G(x) => write!(f, "BaseStation 5G({})", x),
                NodeClass::Controller(x) => write!(f, "Controller ({})", x),
            }
        }
    }

    impl PartialEq for NodeClass {
        fn eq(&self, other: &Self) -> bool {
            self.as_u32() == other.as_u32()
        }
    }

    impl Display for NodeClass {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NodeClass::None => write!(f, "None"),
                NodeClass::Vehicle5G(_) => write!(f, "Vehicle5G"),
                NodeClass::RSU5G(_) => write!(f, "RSU5G"),
                NodeClass::BaseStation5G(_) => write!(f, "BaseStation5G"),
                NodeClass::Controller(_) => write!(f, "Controller"),
            }
        }
    }

    impl NodeClass {
        pub fn as_u32(&self) -> u32 {
            match self {
                NodeClass::None => 0,
                NodeClass::Vehicle5G(x) => *x as u32,
                NodeClass::RSU5G(x) => *x as u32,
                NodeClass::BaseStation5G(x) => *x as u32,
                NodeClass::Controller(x) => *x as u32,
            }
        }
    }

    impl Tier for NodeClass {
        fn as_i32(&self) -> i32 {
            match self {
                NodeClass::None => 0,
                NodeClass::Vehicle5G(x) => *x,
                NodeClass::RSU5G(x) => *x,
                NodeClass::BaseStation5G(x) => *x,
                NodeClass::Controller(x) => *x,
            }
        }
    }
}

pub mod kind {
    use pavenet_engine::entity::Kind;
    use serde::Deserialize;
    use std::fmt::Display;

    #[derive(Deserialize, Debug, Hash, Copy, Default, Clone, PartialEq, Eq)]
    pub enum NodeType {
        #[default]
        Vehicle = 0,
        RSU,
        BaseStation,
        Controller,
    }

    impl Display for NodeType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NodeType::Vehicle => write!(f, "Vehicle"),
                NodeType::RSU => write!(f, "RSU"),
                NodeType::BaseStation => write!(f, "BaseStation"),
                NodeType::Controller => write!(f, "Controller"),
            }
        }
    }

    impl Kind for NodeType {}
}
