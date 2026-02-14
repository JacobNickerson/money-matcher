use rand::{Rng};
use rand_distr::{Distribution, Exp};

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

/// Determines what time the next event occurs
pub trait RateController {
	/// Randomly samples some distribution and returns the nanoseconds expected between last and current event
	fn next_dt(&mut self, rng: &mut impl Rng) -> u64;
}

pub struct ConstantPoissonRate {
	exp: Exp<f64>, 
}
impl ConstantPoissonRate {
	/// Rate in events per second
	pub fn new(rate: f64) -> Self {
        Self {
			exp: Exp::new(rate).unwrap()
        }
    }
}
impl RateController for ConstantPoissonRate {
    fn next_dt(&mut self, rng: &mut impl Rng) -> u64 {
		let dt_seconds = self.exp.sample(rng);
        (dt_seconds * NANOSECONDS_PER_SECOND as f64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
	use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

	#[test]
	#[should_panic]
	fn negative_rate_errors() {
		ConstantPoissonRate::new(-1.0);
	}

	#[test]
	fn constant_poisson_expected_time() {
		let mut rate_controller = ConstantPoissonRate::new(1000000.0);
		let mut seeded_rng = ChaCha8Rng::seed_from_u64(5);
		let mut sum: u64 = 0;
		for _ in 0..1000000 {
			sum += rate_controller.next_dt(&mut seeded_rng);
		}
		let total_elapsed_time = sum as f64 / NANOSECONDS_PER_SECOND as f64;
		const EXPECTED_RUNTIME: f64 = 1.0;
		const PRECISION: f64 = 1.0;
		assert!((total_elapsed_time - EXPECTED_RUNTIME).abs() < PRECISION);
	}
}