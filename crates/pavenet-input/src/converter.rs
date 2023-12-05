pub(crate) mod series {
    use pavenet_core::mobility::road::RoadId;
    use pavenet_core::mobility::velocity::Velocity;
    use pavenet_engine::bucket::TimeMS;
    use pavenet_engine::node::NodeId;
    use polars::series::Series;

    pub(crate) fn to_i64_vec(series: &Series) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<i64> = series_to_option_vec.iter().filter_map(|x| *x).collect();
        Ok(option_vec_to_vec)
    }

    pub(crate) fn to_f32_vec(series: &Series) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
        let option_vec_to_vec: Vec<f32> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| x as f32)
            .collect();
        Ok(option_vec_to_vec)
    }

    pub(crate) fn to_nodeid_vec(
        series: &Series,
    ) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<NodeId> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(NodeId::from)
            .collect::<Vec<NodeId>>();
        Ok(option_vec_to_vec)
    }

    pub(crate) fn to_timestamp_vec(
        series: &Series,
    ) -> Result<Vec<TimeMS>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<TimeMS> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(TimeMS::from)
            .collect();
        Ok(option_vec_to_vec)
    }

    pub(crate) fn to_roadid_vec(
        series: &Series,
    ) -> Result<Vec<RoadId>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<RoadId> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(RoadId::from)
            .collect();
        Ok(option_vec_to_vec)
    }

    pub(crate) fn to_velocity_vec(
        series: &Series,
    ) -> Result<Vec<Velocity>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
        let option_vec_to_vec: Vec<Velocity> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(Velocity::from)
            .collect();
        Ok(option_vec_to_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::series::*;
    use pavenet_core::mobility::road::RoadId;
    use pavenet_core::mobility::velocity::Velocity;
    use pavenet_engine::bucket::TimeMS;
    use pavenet_engine::node::NodeId;
    use polars::prelude::{NamedFrom, Series};

    #[test]
    fn test_to_i64_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let result = to_i64_vec(&series).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_to_f32_vec() {
        let series = Series::new("a", &[Some(1f64), Some(2f64), Some(3f64)]);
        let result = to_f32_vec(&series).unwrap();
        assert_eq!(result, vec![1f32, 2f32, 3f32]);
    }

    #[test]
    fn test_to_nodeid_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let result = to_nodeid_vec(&series).unwrap();
        assert_eq!(
            result,
            vec![NodeId::from(1i64), NodeId::from(2i64), NodeId::from(3i64)]
        );
    }

    #[test]
    fn test_to_timestamp_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let result = to_timestamp_vec(&series).unwrap();
        assert_eq!(
            result,
            vec![TimeMS::from(1i64), TimeMS::from(2i64), TimeMS::from(3i64)]
        );
    }

    #[test]
    fn test_to_roadid_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let result = to_roadid_vec(&series).unwrap();
        assert_eq!(
            result,
            vec![RoadId::from(1i64), RoadId::from(2i64), RoadId::from(3i64)]
        );
    }

    #[test]
    fn test_to_velocity_vec() {
        let series = Series::new("a", &[Some(1f64), Some(2f64), Some(3f64)]);
        let result = to_velocity_vec(&series).unwrap();
        assert_eq!(
            result,
            vec![
                Velocity::from(1f64),
                Velocity::from(2f64),
                Velocity::from(3f64)
            ]
        );
    }
}
