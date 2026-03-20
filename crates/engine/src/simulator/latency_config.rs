use crate::simulator::simulator::SimTime;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct LatencyConfig {
    pub latency: SimTime,
    pub jitter: SimJitter,
}
