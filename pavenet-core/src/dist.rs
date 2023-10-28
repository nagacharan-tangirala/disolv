use anyhow::Result;
use rand_distr::{Distribution, Exp, Gamma, LogNormal, Normal, Uniform};
use rand_pcg::Pcg64Mcg;
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub enum DistType {
    Uniform(Uniform<f32>),
    Normal(Normal<f32>),
    LogNormal(LogNormal<f32>),
    Exponential(Exp<f32>),
    Gamma(Gamma<f32>),
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize)]
pub struct DistParams {
    pub dist_name: String,
    pub seed: Option<u128>,
    pub mean: Option<f32>,
    pub std_dev: Option<f32>,
    pub location: Option<f32>,
    pub scale: Option<f32>,
    pub shape: Option<f32>,
    pub rate: Option<f32>,
    pub min: Option<f32>,
    pub max: Option<f32>,
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
        let min = dist_params.min.ok_or(anyhow::anyhow!("Missing min"))?;
        let max = dist_params.max.ok_or(anyhow::anyhow!("Missing max"))?;
        Ok(Self::Uniform(Uniform::new(min, max)))
    }

    fn build_normal(dist_params: DistParams) -> Result<Self> {
        let mean = dist_params.mean.ok_or(anyhow::anyhow!("Missing mean"))?;
        let std_dev = dist_params
            .std_dev
            .ok_or(anyhow::anyhow!("Missing std_dev"))?;
        Ok(Self::Normal(Normal::new(mean, std_dev)?))
    }

    fn build_lognormal(dist_params: DistParams) -> Result<Self> {
        let mean = dist_params.mean.ok_or(anyhow::anyhow!("Missing mean"))?;
        let std_dev = dist_params
            .std_dev
            .ok_or(anyhow::anyhow!("Missing std_dev"))?;
        Ok(Self::LogNormal(LogNormal::new(mean, std_dev)?))
    }

    fn build_exponential(dist_params: DistParams) -> Result<Self> {
        let rate = dist_params.rate.ok_or(anyhow::anyhow!("Missing rate"))?;
        Ok(Self::Exponential(Exp::new(rate)?))
    }

    fn build_gamma(dist_params: DistParams) -> Result<Self> {
        let shape = dist_params.shape.ok_or(anyhow::anyhow!("Missing shape"))?;
        let scale = dist_params.scale.ok_or(anyhow::anyhow!("Missing scale"))?;
        Ok(Self::Gamma(Gamma::new(shape, scale)?))
    }
}

#[derive(Debug, Clone)]
pub struct RngSampler {
    pub dist: DistType,
    pub rng: Pcg64Mcg,
}

impl RngSampler {
    pub fn new(dist_name: String, params: DistParams) -> Self {
        let seed = params.seed.unwrap_or(0u128);
        let dist = DistType::new(dist_name, params);
        Self {
            dist,
            rng: Pcg64Mcg::new(seed),
        }
    }

    pub fn sample(&mut self) -> f32 {
        match self.dist {
            DistType::Uniform(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Normal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::LogNormal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Exponential(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Gamma(ref mut dist) => dist.sample(&mut self.rng),
        }
    }
}
