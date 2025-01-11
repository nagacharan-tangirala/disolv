use rand_distr::{Dirichlet, Distribution, Exp, Gamma, LogNormal, Normal, Uniform};
use rand_pcg::Pcg64Mcg;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum DistType {
    Uniform(Uniform<f64>),
    Normal(Normal<f64>),
    LogNormal(LogNormal<f64>),
    Exponential(Exp<f64>),
    Gamma(Gamma<f64>),
    Dirichlet(Dirichlet<f64>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistParams {
    pub dist_name: String,
    pub seed: Option<u64>,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub location: Option<f64>,
    pub scale: Option<f64>,
    pub shape: Option<f64>,
    pub rate: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub weights: Option<Vec<f64>>,
}

impl DistType {
    pub fn new(params: DistParams) -> Self {
        match params.dist_name.to_lowercase().as_str() {
            "uniform" => match Self::build_uniform(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid uniform distribution parameters"),
            },
            "normal" => match Self::build_normal(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid normal distribution parameters"),
            },
            "lognormal" => match Self::build_log_normal(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid lognormal distribution parameters"),
            },
            "exponential" => match Self::build_exponential(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid exp. distribution parameters"),
            },
            "gamma" => match Self::build_gamma(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid gamma distribution parameters"),
            },
            "dirichlet" => match Self::build_dirichlet(params) {
                Ok(dist) => dist,
                Err(_) => panic!("Invalid dirichlet distribution parameters"),
            },
            _ => panic!(
                "Invalid distribution name. Supported values are:\
                     uniform, normal, lognormal, exponential, gamma, dirichlet"
            ),
        }
    }

    fn build_uniform(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let min = dist_params.min.ok_or("Missing min")?;
        let max = dist_params.max.ok_or("Missing max")?;
        Ok(Self::Uniform(Uniform::new(min, max)))
    }

    fn build_normal(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let mean = dist_params.mean.ok_or("Missing mean")?;
        let std_dev = dist_params.std_dev.ok_or("Missing std_dev")?;
        Ok(Self::Normal(Normal::new(mean, std_dev)?))
    }

    fn build_log_normal(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let mean = dist_params.mean.ok_or("Missing mean")?;
        let std_dev = dist_params.std_dev.ok_or("Missing std_dev")?;
        Ok(Self::LogNormal(LogNormal::new(mean, std_dev)?))
    }

    fn build_exponential(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let rate = dist_params.rate.ok_or("Missing rate")?;
        Ok(Self::Exponential(Exp::new(rate)?))
    }

    fn build_gamma(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let shape = dist_params.shape.ok_or("Missing shape")?;
        let scale = dist_params.scale.ok_or("Missing scale")?;
        Ok(Self::Gamma(Gamma::new(shape, scale)?))
    }

    fn build_dirichlet(dist_params: DistParams) -> Result<Self, Box<dyn std::error::Error>> {
        let alpha = dist_params.weights.ok_or("Missing weights")?;
        Ok(Self::Dirichlet(Dirichlet::new(alpha.as_slice())?))
    }
}

#[derive(Debug, Clone)]
pub struct UnitSampler {
    pub dist: DistType,
    pub rng: Pcg64Mcg,
}

impl UnitSampler {
    pub fn new(params: DistParams) -> Self {
        let seed: u128 = params.seed.unwrap_or(0) as u128;
        let dist = DistType::new(params);
        Self {
            dist,
            rng: Pcg64Mcg::new(seed),
        }
    }

    pub fn sample(&mut self) -> f64 {
        match self.dist {
            DistType::Uniform(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Normal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::LogNormal(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Exponential(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Gamma(ref mut dist) => dist.sample(&mut self.rng),
            DistType::Dirichlet(_) => {
                panic!("Cannot sample single values from Dirichlet")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeriesSampler {
    pub dist: DistType,
    pub rng: Pcg64Mcg,
}

impl SeriesSampler {
    pub fn new(params: DistParams) -> Self {
        let seed: u128 = params.seed.unwrap_or(0) as u128;
        let dist = DistType::new(params);
        Self {
            dist,
            rng: Pcg64Mcg::new(seed),
        }
    }

    pub fn sample(&mut self) -> Vec<f64> {
        match self.dist {
            DistType::Dirichlet(ref mut dirichlet) => dirichlet.sample(&mut self.rng),
            _ => unimplemented!("Only dirichlet series can be sampled"),
        }
    }
}
