use krabmaga::{addplot, plot, PlotData, DATA};
use pavenet_core::entity::kind::NodeType;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::result::Resultant;
use serde::Deserialize;

pub enum PlotType {
    NodeCounts,
    DataSizes,
}
