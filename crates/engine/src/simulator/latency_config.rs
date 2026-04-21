use crate::cli_args::Args;
use crate::simulator::SimTime;
use clap::ValueEnum;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum JitterKind {
    None,
    Uniform,
    Normal,
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

#[derive(Clone, Copy, Debug)]
pub struct LatencyConfig {
    pub latency: SimTime,
    pub jitter: SimJitter,
}
