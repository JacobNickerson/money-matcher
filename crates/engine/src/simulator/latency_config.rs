use crate::cli_args::Args;
use crate::simulator::SimTime;
use clap::ValueEnum;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

/// Enum denoting the type of distribution used for sampling jitter. Used for selecting distribution from command-line args.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum JitterKind {
    None,
    Uniform,
    Normal,
}
/// Enum that represents simulated jitter. Wraps around different random distributions and provides a method to sample from the held
/// distribution.
#[derive(Clone, Copy, Debug)]
pub enum SimJitter {
    None,
    Uniform(Uniform<u64>),
    Normal(Normal<f64>),
}
impl SimJitter {
    /// Sample the held distribution for jitter, normal distribution is floored to zero
    pub fn sample(&self, rng: &mut impl Rng) -> SimTime {
        match self {
            SimJitter::None => 0,
            SimJitter::Uniform(dist) => dist.sample(rng),
            SimJitter::Normal(dist) => dist.sample(rng).floor().round() as u64,
        }
    }
}

impl From<&Args> for SimJitter {
    fn from(args: &Args) -> Self {
        match args.sim_jitter_type {
            JitterKind::None => SimJitter::None,
            JitterKind::Uniform => SimJitter::Uniform(
                Uniform::new_inclusive(args.low.unwrap(), args.high.unwrap()).unwrap(),
            ),
            JitterKind::Normal => {
                SimJitter::Normal(Normal::new(args.mean.unwrap(), args.std_dev.unwrap()).unwrap())
            }
        }
    }
}

/// Struct containing simulated latency effects
#[derive(Clone, Copy, Debug)]
pub struct LatencyConfig {
    pub latency: SimTime,
    pub jitter: SimJitter,
}
