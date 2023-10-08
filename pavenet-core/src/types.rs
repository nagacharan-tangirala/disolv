pub use super::named::ids::node::NodeId;
pub use super::named::ids::road::RoadId;
pub use super::named::order::Order;
pub use super::named::ts::TimeStamp;
pub use super::named::velocity::Velocity;

pub type PowerTimes = (Vec<TimeStamp>, Vec<TimeStamp>); // (on times, off times)
