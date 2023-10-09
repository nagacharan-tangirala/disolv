use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub enum MobilityType {
    #[default]
    Stationery,
    Mobile,
}

#[derive(Deserialize, Clone, Debug, Copy)]
pub enum TransferMode {
    UDT,
    BDT,
}

#[derive(Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Image,
    Video,
    Lidar2D,
    Lidar3D,
    Radar,
    Status,
}

#[derive(Deserialize, Debug, Copy, Default, Clone, PartialEq, Eq)]
pub enum NodeType {
    #[default]
    Vehicle = 0,
    RSU,
    BaseStation,
    Controller,
}
