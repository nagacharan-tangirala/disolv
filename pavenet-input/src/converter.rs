pub(crate) mod series {
    use pavenet_core::mobility::road::RoadId;
    use pavenet_core::mobility::velocity::Velocity;

    use pavenet_core::bucket::TimeS;
    use pavenet_core::entity::id::NodeId;
    use polars::prelude::Series;

    pub(crate) fn to_i64_vec(series: &Series) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<i64> = series_to_option_vec.iter().filter_map(|x| *x).collect();
        return Ok(option_vec_to_vec);
    }

    pub(crate) fn to_f32_vec(series: &Series) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
        let option_vec_to_vec: Vec<f32> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| x as f32)
            .collect();
        return Ok(option_vec_to_vec);
    }

    pub(crate) fn to_nodeid_vec(
        series: &Series,
    ) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<NodeId> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| NodeId::from(x))
            .collect::<Vec<NodeId>>();
        return Ok(option_vec_to_vec);
    }

    pub(crate) fn to_timestamp_vec(
        series: &Series,
    ) -> Result<Vec<TimeS>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<TimeS> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| TimeS::from(x))
            .collect();
        return Ok(option_vec_to_vec);
    }

    pub(crate) fn to_roadid_vec(
        series: &Series,
    ) -> Result<Vec<RoadId>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<i64>> = series.i64()?.to_vec();
        let option_vec_to_vec: Vec<RoadId> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| RoadId::from(x))
            .collect();
        return Ok(option_vec_to_vec);
    }

    pub(crate) fn to_velocity_vec(
        series: &Series,
    ) -> Result<Vec<Velocity>, Box<dyn std::error::Error>> {
        let series_to_option_vec: Vec<Option<f64>> = series.f64()?.to_vec();
        let option_vec_to_vec: Vec<Velocity> = series_to_option_vec
            .iter()
            .filter_map(|x| *x)
            .map(|x| Velocity::from(x))
            .collect();
        return Ok(option_vec_to_vec);
    }
}

pub(crate) mod list_series {
    use pavenet_core::bucket::TimeS;
    use pavenet_core::entity::id::NodeId;
    use polars::prelude::Series;
    use std::slice;

    pub(crate) fn to_vec_of_nodeid_vec(
        list_series: &Series,
    ) -> Result<Vec<Vec<NodeId>>, Box<dyn std::error::Error>> {
        let mut result_vec: Vec<Vec<NodeId>> = Vec::with_capacity(list_series.len());
        for n in list_series.iter() {
            let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
            let x_vec = x.explode()?.i64()?.to_vec();
            let x_vec: Vec<NodeId> = x_vec
                .iter()
                .filter_map(|x| *x)
                .map(|x| NodeId::from(x))
                .collect();
            result_vec.push(x_vec);
        }
        Ok(result_vec)
    }

    pub(crate) fn to_vec_of_f32_vec(
        list_series: &Series,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let mut result_vec: Vec<Vec<f32>> = Vec::with_capacity(list_series.len());
        for n in list_series.iter() {
            let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
            let x_vec = x.explode()?.f64()?.to_vec();
            let x_vec: Vec<f32> = x_vec.iter().filter_map(|x| *x).map(|x| x as f32).collect();
            result_vec.push(x_vec);
        }
        Ok(result_vec)
    }

    pub(crate) fn to_vec_of_i64_vec(
        list_series: &Series,
    ) -> Result<Vec<Vec<i64>>, Box<dyn std::error::Error>> {
        let mut result_vec: Vec<Vec<i64>> = Vec::with_capacity(list_series.len());
        for n in list_series.iter() {
            let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
            let x_vec = x.explode()?.i64()?.to_vec();
            let x_vec: Vec<i64> = x_vec.iter().filter_map(|x| *x).collect();
            result_vec.push(x_vec);
        }
        Ok(result_vec)
    }

    pub(crate) fn to_vec_of_timestamp_vec(
        list_series: &Series,
    ) -> Result<Vec<Vec<TimeS>>, Box<dyn std::error::Error>> {
        let mut result_vec: Vec<Vec<TimeS>> = Vec::with_capacity(list_series.len());
        for n in list_series.iter() {
            let x = Series::from_any_values("a", slice::from_ref(&n), true)?;
            let x_vec = x.explode()?.i64()?.to_vec();
            let x_vec: Vec<TimeS> = x_vec
                .iter()
                .filter_map(|x| *x)
                .map(|x| TimeS::from(x))
                .collect();
            result_vec.push(x_vec);
        }
        Ok(result_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::list_series::*;
    use super::series::*;
    use pavenet_core::bucket::TimeS;
    use pavenet_core::entity::id::NodeId;
    use pavenet_core::mobility::road::RoadId;
    use pavenet_core::mobility::velocity::Velocity;
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
            vec![TimeS::from(1i64), TimeS::from(2i64), TimeS::from(3i64)]
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

    #[test]
    fn test_to_vec_of_nodeid_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let series2 = Series::new("a", &[Some(4i64), Some(5i64), Some(6i64)]);
        let list_series = Series::new("a", &[Some(series), Some(series2)]);
        println!("{:?}", list_series);
        let result = to_vec_of_nodeid_vec(&list_series).unwrap();
        assert_eq!(
            result,
            vec![
                vec![NodeId::from(1i64), NodeId::from(2i64), NodeId::from(3i64)],
                vec![NodeId::from(4i64), NodeId::from(5i64), NodeId::from(6i64)]
            ]
        );
    }

    #[test]
    fn test_to_vec_of_f32_vec() {
        let series = Series::new("a", &[Some(1f64), Some(2f64), Some(3f64)]);
        let series2 = Series::new("a", &[Some(4f64), Some(5f64), Some(6f64)]);
        let list_series = Series::new("a", &[Some(series), Some(series2)]);
        println!("{:?}", list_series);
        let result = to_vec_of_f32_vec(&list_series).unwrap();
        assert_eq!(result, vec![vec![1f32, 2f32, 3f32], vec![4f32, 5f32, 6f32]]);
    }

    #[test]
    fn test_to_vec_of_i64_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let series2 = Series::new("a", &[Some(4i64), Some(5i64), Some(6i64)]);
        let list_series = Series::new("a", &[Some(series), Some(series2)]);
        println!("{:?}", list_series);
        let result = to_vec_of_i64_vec(&list_series).unwrap();
        assert_eq!(result, vec![vec![1i64, 2i64, 3i64], vec![4i64, 5i64, 6i64]]);
    }

    #[test]
    fn test_to_vec_of_timestamp_vec() {
        let series = Series::new("a", &[Some(1i64), Some(2i64), Some(3i64)]);
        let series2 = Series::new("a", &[Some(4i64), Some(5i64), Some(6i64)]);
        let list_series = Series::new("a", &[Some(series), Some(series2)]);
        println!("{:?}", list_series);
        let result = to_vec_of_timestamp_vec(&list_series).unwrap();
        assert_eq!(
            result,
            vec![
                vec![TimeS::from(1i64), TimeS::from(2i64), TimeS::from(3i64)],
                vec![TimeS::from(4i64), TimeS::from(5i64), TimeS::from(6i64)]
            ]
        );
    }
}
