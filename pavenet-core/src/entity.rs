use crate::entity::class::Class;
use crate::entity::id::NodeId;
use crate::entity::kind::NodeType;
use crate::entity::order::Order;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub node_class: Class,
    pub order: Order,
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
    pub enum NodeClass {
        #[default]
        None = 0,
        Vehicle5G = 1,
        RSU5G = 2,
        BaseStation5G = 3,
        Controller = 4,
    }

    impl Display for NodeClass {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NodeClass::None => write!(f, "None"),
                NodeClass::Vehicle5G => write!(f, "Vehicle5G"),
                NodeClass::BaseStation5G => write!(f, "BaseStation5G"),
                NodeClass::RSU5G => write!(f, "RSU5G"),
                NodeClass::Controller => write!(f, "Controller"),
            }
        }
    }

    impl NodeClass {
        pub fn as_u32(&self) -> u32 {
            match self {
                NodeClass::None => 0,
                NodeClass::Vehicle5G => 1,
                NodeClass::BaseStation5G => 2,
                NodeClass::RSU5G => 3,
                NodeClass::Controller => 4,
            }
        }
    }

    impl Tier for NodeClass {
        fn as_i32(&self) -> i32 {
            match self {
                NodeClass::None => 0,
                NodeClass::Vehicle5G => 1,
                NodeClass::BaseStation5G => 2,
                NodeClass::RSU5G => 3,
                NodeClass::Controller => 4,
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

pub mod class {
    use serde::Deserialize;
    use std::fmt::Display;

    #[derive(Debug, Deserialize, Hash, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Class(u32);

    impl Display for Class {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:03}", self.0)
        }
    }

    impl From<u32> for Class {
        fn from(value: u32) -> Self {
            Self(value)
        }
    }

    impl Class {
        pub fn as_u32(&self) -> u32 {
            self.0
        }
    }
}
