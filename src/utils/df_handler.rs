use krabmaga::hashbrown::HashMap;
use polars::prelude::{col, lit, IntoLazy};
use polars_core::frame::DataFrame;
use polars_core::prelude::Series;

pub struct VehicleTraceHandler {
    trace_df: DataFrame,
}

impl VehicleTraceHandler {
    pub fn new(trace_df: DataFrame) -> Self {
        Self { trace_df }
    }

    #[rustfmt::skip]
    pub fn prepare_trace_dfs(
        &mut self,
    ) -> Result<HashMap<i64, DataFrame>, Box<dyn std::error::Error>> {

        let filtered_df = self.trace_df
            .clone()
            .lazy()
            .groupby(
                [col("vehicle_id")]
            )
            .agg(
                vec![
                    col("time"),
                    col("x"),
                    col("y"),
                ]
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
            .collect().unwrap();

        let vehicle_series: &Series = match filtered_df.columns(["vehicle_id"])?.get(0) {
            Some(series) => *series,
            None => return Err("No vehicle_id column found".into()),
        };

        let vehicle_ids = vehicle_series.i64()?.to_vec().iter()
            .map(|x| x.unwrap_or(0) ).collect::<Vec<i64>>();
        if vehicle_ids.contains(&0) {
            return Err("Vehicle ids should not be 0".into());
        }

        let mut vehicle_dfs: HashMap<i64, DataFrame> = HashMap::new();
        for vehicle_id in vehicle_ids.iter() {
            let vehicle_df = filtered_df
                .clone()
                .lazy()
                .filter(col("vehicle_id").eq(lit(*vehicle_id)))
                .collect()
                .unwrap();
            vehicle_dfs.insert(*vehicle_id, vehicle_df);
        }
        return Ok(vehicle_dfs);
    }
}

pub struct ActivationHandler {
    activation_df: DataFrame,
    device_id: String,
}

impl ActivationHandler {
    pub fn new(activation_df: DataFrame, device_id: &str) -> Self {
        Self {
            activation_df,
            device_id: device_id.to_string(),
        }
    }

    pub fn prepare_device_activations(
        &mut self,
    ) -> Result<HashMap<i64, (Vec<i64>, Vec<i64>)>, Box<dyn std::error::Error>> {
        let filtered_df = self
            .activation_df
            .clone()
            .lazy()
            .groupby([col(&self.device_id)])
            .agg(
                vec![col("activation"), col("deactivation")]
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
            .collect()
            .unwrap();

        let device_id_series: &Series = match filtered_df.columns([&self.device_id])?.get(0) {
            Some(series) => *series,
            None => return Err("Wrong column name given".into()),
        };

        let device_ids = device_id_series
            .i64()?
            .to_vec()
            .iter()
            .map(|x| x.unwrap_or(0))
            .collect::<Vec<i64>>();
        if device_ids.contains(&0) {
            return Err("Device ids should not be 0".into());
        }

        let mut activation_dfs: HashMap<i64, (Vec<i64>, Vec<i64>)> = HashMap::new();
        for device_id in device_ids.iter() {
            let device_df = filtered_df
                .clone()
                .lazy()
                .filter(col(&self.device_id).eq(lit(*device_id)))
                .collect()
                .unwrap();
            let activation_series: &Series = match device_df.columns(["activation"])?.get(0) {
                Some(series) => *series,
                None => return Err("No activation column found".into()),
            };
            let deactivation_series: &Series = match device_df.columns(["deactivation"])?.get(0) {
                Some(series) => *series,
                None => return Err("No deactivation column found".into()),
            };
            let activations = activation_series
                .i64()?
                .to_vec()
                .iter()
                .map(|x| x.unwrap_or(0))
                .collect::<Vec<i64>>();
            let deactivations = deactivation_series
                .i64()?
                .to_vec()
                .iter()
                .map(|x| x.unwrap_or(0))
                .collect::<Vec<i64>>();
            activation_dfs.insert(*device_id, (activations, deactivations));
        }

        return Ok(activation_dfs);
    }
}
