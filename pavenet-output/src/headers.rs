pub(crate) mod node_pos {
    use pavenet_core::mobility::MapState;
    use pavenet_engine::bucket::TimeS;
    use pavenet_engine::entity::NodeId;
    use pavenet_engine::result::Resultant;
    use serde::Serialize;

    #[derive(Default, Clone, Copy, Debug, Serialize)]
    pub struct NodePosition {
        pub(crate) time_step: u32,
        pub(crate) node_id: u32,
        pub(crate) x: f32,
        pub(crate) y: f32,
    }

    impl NodePosition {
        pub fn from_data(time_step: TimeS, node_id: NodeId, map_state: &MapState) -> Self {
            Self {
                time_step: time_step.as_u32(),
                node_id: node_id.as_u32(),
                x: map_state.pos.x,
                y: map_state.pos.y,
            }
        }
    }

    impl Resultant for NodePosition {}

    pub(crate) const TIME_STEP: &str = "time_step";
    pub(crate) const NODE_ID: &str = "node_id";
    pub(crate) const COORD_X: &str = "x";
    pub(crate) const COORD_Y: &str = "y";
    pub(crate) const VELOCITY: &str = "velocity";
}

pub(crate) mod payload_tx {
    use pavenet_core::payload::DPayload;
    use pavenet_core::response::TransferMetrics;
    use pavenet_engine::bucket::TimeS;
    use pavenet_engine::payload::PayloadStatus;
    use pavenet_engine::radio::Metric;
    use pavenet_engine::result::Resultant;
    use serde::Serialize;

    #[derive(Default, Clone, Copy, Debug, Serialize)]
    pub struct DataTx {
        pub(crate) time_step: u32,
        pub(crate) node_id: u32,
        pub(crate) selected_node: u32,
        pub(crate) distance: f32,
        pub(crate) data_size: f32,
        pub(crate) data_count: u32,
    }

    impl DataTx {
        pub fn from_data(time_step: TimeS, payload: &DPayload) -> Self {
            Self {
                time_step: time_step.as_u32(),
                node_id: payload.content.node_info.id.as_u32(),
                selected_node: payload.metadata.tx_info.selected_link.target.as_u32(),
                distance: payload
                    .metadata
                    .tx_info
                    .selected_link
                    .properties
                    .distance
                    .unwrap_or_default(),
                data_size: payload.metadata.total_size,
                data_count: payload.metadata.total_count,
            }
        }
    }

    impl Resultant for DataTx {}

    pub(crate) const TIME_STEP: &str = "time_step";
    pub(crate) const NODE_ID: &str = "node_id";
    pub(crate) const SELECTED_NODE: &str = "selected_node";
    pub(crate) const DISTANCE: &str = "distance";
    pub(crate) const TARGET_ID: &str = "target_id";
    pub(crate) const DATA_SIZE: &str = "data_size";
    pub(crate) const DATA_COUNT: &str = "data_count";
    pub(crate) const STATUS: &str = "status";
    pub(crate) const LATENCY: &str = "latency";
}

pub(crate) mod tx_status {}

pub(crate) mod data_rx {
    use pavenet_core::radio::stats::InDataStats;
    use pavenet_engine::bucket::TimeS;
    use pavenet_engine::entity::NodeId;
    use pavenet_engine::radio::Metric;
    use pavenet_engine::result::Resultant;
    use serde::Serialize;

    #[derive(Default, Clone, Copy, Debug, Serialize)]
    pub struct DataRx {
        pub(crate) time_step: u32,
        pub(crate) node_id: u32,
        pub(crate) attempted_in_node_count: u32,
        pub(crate) attempted_in_data_size: f32,
        pub(crate) attempted_in_data_count: u32,
        pub(crate) feasible_in_node_count: u32,
        pub(crate) feasible_in_data_size: f32,
        pub(crate) feasible_in_data_count: u32,
        pub(crate) avg_latency: f32,
    }

    impl DataRx {
        pub fn from_data(time_step: TimeS, node_id: NodeId, in_data_stats: &InDataStats) -> Self {
            Self {
                time_step: time_step.as_u32(),
                node_id: node_id.as_u32(),
                attempted_in_node_count: in_data_stats.attempted.node_count,
                attempted_in_data_size: in_data_stats.attempted.data_size,
                attempted_in_data_count: in_data_stats.attempted.data_count,
                feasible_in_node_count: in_data_stats.feasible.node_count,
                feasible_in_data_size: in_data_stats.feasible.data_size,
                feasible_in_data_count: in_data_stats.feasible.data_count,
                avg_latency: in_data_stats.avg_latency.as_f32(),
            }
        }
    }

    impl Resultant for DataRx {}

    pub(crate) const TIME_STEP: &str = "time_step";
    pub(crate) const NODE_ID: &str = "node_id";
    pub(crate) const ATTEMPTED_IN_NODE_COUNT: &str = "attempted_in_node_count";
    pub(crate) const ATTEMPTED_IN_DATA_SIZE: &str = "attempted_in_data_size";
    pub(crate) const ATTEMPTED_IN_DATA_COUNT: &str = "attempted_in_data_count";
    pub(crate) const FEASIBLE_IN_NODE_COUNT: &str = "feasible_in_node_count";
    pub(crate) const FEASIBLE_IN_DATA_SIZE: &str = "feasible_in_data_size";
    pub(crate) const FEASIBLE_IN_DATA_COUNT: &str = "feasible_in_data_count";
    pub(crate) const AVG_LATENCY: &str = "avg_latency";
}
