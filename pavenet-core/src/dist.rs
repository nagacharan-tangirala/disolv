use anyhow::Result;
use rand::distributions::Distribution;
use serde::Deserialize;
use statrs::distribution::{Exp, Gamma, LogNormal, Normal, Uniform};

#[derive(Debug, Clone, Copy)]
pub enum DistType {
    Uniform(Uniform),
    Normal(Normal),
    LogNormal(LogNormal),
    Exponential(Exp),
    Gamma(Gamma),
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize)]
pub struct DistParams {
    pub dist_name: String,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub location: Option<f64>,
    pub scale: Option<f64>,
    pub shape: Option<f64>,
    pub rate: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DistType {
    pub fn new(dist_name: String, params: DistParams) -> Self {
        match dist_name.as_str() {
            "uniform" => match Self::build_uniform(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid distribution parameters"),
            },
            "normal" => match Self::build_normal(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid distribution parameters"),
            },
            "lognormal" => match Self::build_lognormal(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid distribution parameters"),
            },
            "exponential" => match Self::build_exponential(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid distribution parameters"),
            },
            "gamma" => match Self::build_gamma(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid distribution parameters"),
            },
            _ => panic!("Invalid distribution name"),
        }
    }

    fn build_uniform(dist_params: DistParams) -> Result<Self> {
        let min: f64 = dist_params.min.ok_or(anyhow::anyhow!("Missing min"))?;
        let max: f64 = dist_params.max.ok_or(anyhow::anyhow!("Missing max"))?;
        Ok(Self::Uniform(Uniform::new(min, max)?))
    }

    fn build_normal(dist_params: DistParams) -> Result<Self> {
        let mean: f64 = dist_params.mean.ok_or(anyhow::anyhow!("Missing mean"))?;
        let std_dev: f64 = dist_params
            .std_dev
            .ok_or(anyhow::anyhow!("Missing std_dev"))?;
        Ok(Self::Normal(Normal::new(mean, std_dev)?))
    }

    fn build_lognormal(dist_params: DistParams) -> Result<Self> {
        let mean: f64 = dist_params.mean.ok_or(anyhow::anyhow!("Missing mean"))?;
        let std_dev: f64 = dist_params
            .std_dev
            .ok_or(anyhow::anyhow!("Missing std_dev"))?;
        Ok(Self::LogNormal(LogNormal::new(mean, std_dev)?))
    }

    fn build_exponential(dist_params: DistParams) -> Result<Self> {
        let rate: f64 = dist_params.rate.ok_or(anyhow::anyhow!("Missing rate"))?;
        Ok(Self::Exponential(Exp::new(rate)?))
    }

    fn build_gamma(dist_params: DistParams) -> Result<Self> {
        let shape: f64 = dist_params.shape.ok_or(anyhow::anyhow!("Missing shape"))?;
        let scale: f64 = dist_params.scale.ok_or(anyhow::anyhow!("Missing scale"))?;
        Ok(Self::Gamma(Gamma::new(shape, scale)?))
    }
}

#[derive(Debug, Clone)]
pub struct RngSampler {
    pub dist: DistType,
    pub rng: rand::rngs::ThreadRng,
}

impl RngSampler {
    pub fn new(dist_name: String, params: DistParams) -> Self {
        let dist = DistType::new(dist_name, params);
        Self {
            dist,
            rng: rand::thread_rng(),
        }
    }

    pub fn sample(&mut self) -> f64 {
        match self.dist {
            DistType::Uniform(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Normal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::LogNormal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Exponential(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Gamma(ref mut dist) => dist.sample(&mut self.rng),
        }
    }
}
