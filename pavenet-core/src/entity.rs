use crate::entity::class::NodeClass;
use crate::entity::id::NodeId;
use crate::entity::kind::NodeType;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: NodeClass,
}

pub mod id {
    use pavenet_engine::entity::Identifier;
    use serde::Deserialize;
    use std::fmt::Display;
    use std::str::FromStr;

    #[derive(Deserialize, Default, Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct NodeId(u32);

    impl Display for NodeId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:09}", self.0)
        }
    }

    impl FromStr for NodeId {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let id = s.parse::<u32>()?;
            Ok(Self(id))
        }
    }

    impl From<u32> for NodeId {
        fn from(f: u32) -> Self {
            Self(f)
        }
    }

    impl From<i64> for NodeId {
        fn from(f: i64) -> Self {
            Self(f as u32)
        }
    }

    impl NodeId {
        pub fn as_i64(&self) -> i64 {
            self.0 as i64
        }
    }

    impl Identifier for NodeId {
        fn as_u32(&self) -> u32 {
            self.0
        }
    }
}

pub mod class {
    use pavenet_engine::entity::Tier;
    use serde::Deserialize;
    use std::fmt::Display;

    #[derive(Deserialize, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(tag = "class_name", content = "class_order")]
    pub enum NodeClass {
        #[default]
        None,
        Vehicle5G(i32),
        RSU5G(i32),
        BaseStation5G(i32),
        Controller(i32),
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
