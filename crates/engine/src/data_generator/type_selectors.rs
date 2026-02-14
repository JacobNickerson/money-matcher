use crate::lob::order::{OrderSide, OrderType};
use rand::Rng;
use rand_distr::{Distribution, Uniform};

/// Determines the type of the next event (bid/ask, new,modify,cancel)
pub trait TypeSelector {
    /// Randomly samples some distribution and returns a Side and OrderType
    fn sample(&mut self, rng: &mut impl Rng) -> (OrderSide, OrderType);
}

pub struct UniformTypeSelector {
    bid_proportion: f64,
    new_limit_cutoff: f64,
    new_market_cutoff: f64,
    cancel_cutoff: f64,
    side_dist: Uniform<f64>,
    type_dist: Uniform<f64>,
}
impl UniformTypeSelector {
    pub fn new(
        bid_rate: f64,
        new_limit_rate: f64,
        new_market_rate: f64,
        cancel_rate: f64,
        update_rate: f64,
    ) -> Self {
        assert!(bid_rate >= 0.0);
        assert!(new_limit_rate >= 0.0);
        assert!(new_market_rate >= 0.0);
        assert!(cancel_rate >= 0.0);
        assert!(update_rate >= 0.0);
        let type_sum = new_limit_rate + cancel_rate + new_market_rate + update_rate;
        assert!(type_sum > 0.0);
        Self {
            bid_proportion: bid_rate,
            new_limit_cutoff: new_limit_rate,
            new_market_cutoff: new_limit_rate + new_market_rate,
            cancel_cutoff: new_limit_rate + new_market_rate + cancel_rate,
            side_dist: Uniform::new_inclusive(0.0, 1.0).unwrap(),
            type_dist: Uniform::new_inclusive(0.0, type_sum).unwrap(),
        }
    }
}
impl TypeSelector for UniformTypeSelector {
    fn sample(&mut self, rng: &mut impl Rng) -> (OrderSide, OrderType) {
        let num = self.side_dist.sample(rng);
        let order_side = match num <= self.bid_proportion {
            true => OrderSide::Bid,
            false => OrderSide::Ask,
        };
        let sample = self.type_dist.sample(rng);
        if sample < self.new_limit_cutoff {
            (order_side, OrderType::Limit { qty: 0, price: 0 })
        } else if sample < self.new_market_cutoff {
            (order_side, OrderType::Market { qty: 0 })
        } else if sample < self.cancel_cutoff {
            (order_side, OrderType::Cancel)
        } else {
            (
                order_side,
                OrderType::Update {
                    old_id: 0,
                    qty: 0,
                    price: 0,
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    #[should_panic]
    fn bid_rate_must_be_nonnegative() {
        UniformTypeSelector::new(-0.1, 0.0, 0.0, 0.0, 0.0);
    }

    #[test]
    #[should_panic]
    fn limit_rate_must_be_nonnegative() {
        UniformTypeSelector::new(0.1, -0.1, 0.1, 0.1, 0.1);
    }

    #[test]
    #[should_panic]
    fn market_rate_must_be_nonnegative() {
        UniformTypeSelector::new(0.1, 0.1, -0.1, 0.1, 0.1);
    }

    #[test]
    #[should_panic]
    fn cancel_rate_must_be_nonnegative() {
        UniformTypeSelector::new(0.1, 0.1, 0.1, -0.1, 0.1);
    }

    #[test]
    #[should_panic]
    fn modify_rate_must_be_nonnegative() {
        UniformTypeSelector::new(0.1, 0.1, 0.1, 0.1, -0.1);
    }

    #[test]
    #[should_panic]
    fn order_type_total_rate_must_be_nonzero() {
        UniformTypeSelector::new(0.5, 0.0, 0.0, 0.0, 0.0);
    }

    #[test]
    fn side_selection_ratio_approximately_equals_rate() {
        let mut type_selector = UniformTypeSelector::new(0.75, 0.1, 0.1, 0.1, 0.1);
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(1);
        let mut ask_count: i64 = 0;
        let mut bid_count: i64 = 0;
        for _ in 0..1000000 {
            match type_selector.sample(&mut seeded_rng) {
                (OrderSide::Ask, _) => {
                    ask_count += 1;
                }
                (OrderSide::Bid, _) => {
                    bid_count += 1;
                }
            }
        }
        const PRECISION: f64 = 0.025; // NOTE: Picked arbitrarily, lower precision as tradeoff for smaller sample/faster test
        let ratio = bid_count as f64 / ask_count as f64;
        assert!(ratio > 3.0 - PRECISION);
        assert!(ratio < 3.0 + PRECISION);
    }

    #[test]
    fn type_selection_ratio_approximately_equals_rate() {
        let limit_rate = 0.40;
        let market_rate = 0.20;
        let cancel_rate = 0.30;
        let update_rate = 0.10;

        let mut type_selector =
            UniformTypeSelector::new(0.75, limit_rate, market_rate, cancel_rate, update_rate);
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(1);
        let mut limit_count: i64 = 0;
        let mut market_count: i64 = 0;
        let mut cancel_count: i64 = 0;
        let mut update_count: i64 = 0;

        let sample_count = 1000000;
        for _ in 0..sample_count {
            match type_selector.sample(&mut seeded_rng) {
                (_, OrderType::Limit { qty: _, price: _ }) => {
                    limit_count += 1;
                }
                (_, OrderType::Market { qty: _ }) => {
                    market_count += 1;
                }
                (_, OrderType::Cancel) => {
                    cancel_count += 1;
                }
                (
                    _,
                    OrderType::Update {
                        old_id: _,
                        qty: _,
                        price: _,
                    },
                ) => {
                    update_count += 1;
                }
            }
        }
        const PRECISION: f64 = 0.025; // NOTE: Picked arbitrarily, lower precision as tradeoff for smaller sample/faster test
        let limit_ratio = limit_count as f64 / sample_count as f64;
        let market_ratio = market_count as f64 / sample_count as f64;
        let cancel_ratio = cancel_count as f64 / sample_count as f64;
        let update_ratio = update_count as f64 / sample_count as f64;
        assert!((limit_ratio) < limit_rate + PRECISION);
        assert!((limit_ratio) > limit_rate - PRECISION);
        assert!((market_ratio) < market_rate + PRECISION);
        assert!((market_ratio) > market_rate - PRECISION);
        assert!((cancel_ratio) < cancel_rate + PRECISION);
        assert!((cancel_ratio) > cancel_rate - PRECISION);
        assert!((update_ratio) < update_rate + PRECISION);
        assert!((update_ratio) > update_rate - PRECISION);
    }
}
