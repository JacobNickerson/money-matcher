use crate::simulator::simulator::SimTime;
use clap::Subcommand;
use engine::positive_float_parser;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

#[derive(Debug, Subcommand)]
pub enum JitterCfg {
    /// No SimJitter (default)
    None,

    /// SimJitter sampled using uniform distribution
    Uniform {
        /// Inclusive low value of range in nanoseconds
        #[arg(long)]
        low: u64,

        /// Inclusive high value of range in nanoseconds
        #[arg(long)]
        high: u64,
    },

    /// SimJitter sampled using gaussian distribution
    Normal {
        /// Mean value in nanoseconds
        #[arg(long, value_parser = positive_float_parser)]
        mean: f64,

        /// Standard deviation in nanoseconds
        #[arg(long, value_parser = positive_float_parser)]
        std_dev: f64,
    },
}
impl JitterCfg {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            JitterCfg::None => Ok(()),
            JitterCfg::Normal { mean, std_dev } => Ok(()),
            JitterCfg::Uniform { low, high } => {
                if low > high {
                    return Err("uniform jitter: `low` must be <= `high`".into());
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SimJitter {
    None,
    Uniform(Uniform<u64>),
    Normal(Normal<f64>),
}
impl SimJitter {
    pub fn sample(&self, rng: &mut impl Rng) -> SimTime {
        match self {
            SimJitter::None => 0,
            SimJitter::Uniform(dist) => dist.sample(rng),
            SimJitter::Normal(dist) => dist.sample(rng).round() as u64,
        }
    }
}
impl From<Option<JitterCfg>> for SimJitter {
    fn from(cfg: Option<JitterCfg>) -> SimJitter {
        if let Some(cfg) = cfg {
            match cfg.validate() {
                Ok(_) => {}
                Err(msg) => panic!("{}", msg),
            }
            match cfg {
                JitterCfg::None => SimJitter::None,
                JitterCfg::Uniform { low, high } => {
                    SimJitter::Uniform(Uniform::new_inclusive(low, high).unwrap())
                }
                JitterCfg::Normal { mean, std_dev } => {
                    SimJitter::Normal(Normal::new(mean, std_dev).unwrap())
                }
            }
        } else {
            SimJitter::None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LatencyConfig {
    pub latency: SimTime,
    pub jitter: SimJitter,
}
