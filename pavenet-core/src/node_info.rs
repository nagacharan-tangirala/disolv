use crate::node_info::class::Class;
use crate::node_info::id::NodeId;
use crate::node_info::kind::NodeType;
use crate::node_info::order::Order;
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

pub mod order {
    use pavenet_engine::entity::Tier;
    use serde::Deserialize;
    use std::fmt::Display;

    #[derive(Deserialize, Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Order(i32);

    impl Display for Order {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:03}", self.0)
        }
    }

    impl From<i32> for Order {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }

    impl Order {
        pub fn as_u32(&self) -> u32 {
            self.0 as u32
        }
    }

    impl Tier for Order {
        fn as_i32(&self) -> i32 {
            self.0
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