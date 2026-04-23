use fastrand;
use mm_core::lob_core::{
    ClientId, OrderId, OrderQty, Price, Timestamp,
    market_orders::{Order, OrderSide, OrderType},
};
use rand::{Rng, RngExt};
use rand_distr::{
    Distribution, Normal, Uniform,
    uniform::{UniformSampler, UniformUsize},
};

use crate::simulator::SimTime;

/// Generates the next event, also handles selection of price level, memory of orders for cancellation
pub trait OrderGenerator {
    /// Accepts an order type and randomly samples some distribution to generate an Order
    fn generate(
        &mut self,
        client_id: ClientId,
        time_stamp: Timestamp,
        order_variant: (OrderSide, OrderType),
        rng: &mut impl Rng,
    ) -> Order;
}

pub struct GaussianOrderGenerator {
    bid_dist: Normal<f64>,
    ask_dist: Normal<f64>,
    current_time: SimTime,
    order_counter: u64,
    qty_dist: Uniform<OrderQty>,
}
impl GaussianOrderGenerator {
    pub fn new(bid_mean: f64, bid_deviation: f64, ask_mean: f64, ask_deviation: f64) -> Self {
        Self {
            bid_dist: Normal::new(bid_mean, bid_deviation).unwrap(),
            ask_dist: Normal::new(ask_mean, ask_deviation).unwrap(),
            current_time: 0,
            order_counter: 0,
            qty_dist: Uniform::new_inclusive(0, 20).unwrap(), // TODO: Make this configurable
        }
    }
    fn compute_price(&mut self, side: OrderSide, rng: &mut impl Rng) -> Price {
        match side {
            OrderSide::Ask => self.ask_dist.sample(rng) as Price,
            OrderSide::Bid => self.bid_dist.sample(rng) as Price,
        }
    }
    fn get_active_order(&self, rng: &mut impl Rng) -> OrderId {
        let dist = Uniform::new(0, self.order_counter).unwrap();
        dist.sample(rng)
    }
}
impl OrderGenerator for GaussianOrderGenerator {
    fn generate(
        &mut self,
        client_id: ClientId,
        time_stamp: Timestamp,
        order_variant: (OrderSide, OrderType),
        rng: &mut impl Rng,
    ) -> Order {
        let (side, kind) = order_variant;
        let price = self.compute_price(side, rng);
        let qty = self.qty_dist.sample(rng);
        self.order_counter += 1;
        self.current_time += time_stamp;
        match kind {
            OrderType::Limit { .. } => {
                self.order_counter += 1;
                Order::new(
                    client_id,
                    0, // NOTE: Use a junk value, simulator sets this on receipt
                    side,
                    self.current_time,
                    OrderType::Limit { qty, price },
                )
            }
            OrderType::Market { .. } => Order::new(
                client_id,
                0, // NOTE: Use a junk value, simulator sets this on receipt
                side,
                self.current_time,
                OrderType::Market { qty },
            ),
            OrderType::Cancel { .. } => Order::new(
                client_id,
                0,
                side,
                self.current_time,
                OrderType::Cancel {
                    old_id: self.get_active_order(rng),
                },
            ),
            OrderType::Update { .. } => Order::new(
                client_id,
                0, // NOTE: Use a junk value, simulator sets this on receipt
                side,
                self.current_time,
                OrderType::Update {
                    old_id: self.get_active_order(rng),
                    qty,
                    price,
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::vec::Vec;

    use super::*;

    const PRECISION: f64 = 0.025;

    fn generate_limit_orders(count: u64) -> Vec<Order> {
        let mut orders = Vec::new();
        let mut order_gen = GaussianOrderGenerator::new(150.0, 1.0, 50.0, 1.0);
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(0);
        for i in 0..count {
            orders.push(order_gen.generate(
                0,
                i,
                (OrderSide::Bid, OrderType::Limit { qty: 0, price: 0 }),
                &mut seeded_rng,
            ));
        }
        for i in 0..count {
            orders.push(order_gen.generate(
                0,
                i,
                (OrderSide::Ask, OrderType::Limit { qty: 0, price: 0 }),
                &mut seeded_rng,
            ));
        }
        orders
    }

    #[test]
    fn test_price_mean() {
        let orders = generate_limit_orders(1000000);
        let mut bid_total_price: u64 = 0;
        let mut ask_total_price: u64 = 0;
        for order in &orders {
            if let OrderType::Limit { qty: _, price } = order.kind {
                match order.side {
                    OrderSide::Bid => bid_total_price += price as u64,
                    OrderSide::Ask => ask_total_price += price as u64,
                }
            }
        }
        let avg_bid_price = bid_total_price as f64 / 1000000.0;
        let avg_ask_price = ask_total_price as f64 / 1000000.0;
        println!("{avg_bid_price}");
        println!("{avg_ask_price}");
        assert!(avg_bid_price / 150.0 < 1.0 + PRECISION);
        assert!(avg_bid_price / 150.0 > 1.0 - PRECISION);
        assert!(avg_ask_price / 50.0 < 1.0 + PRECISION);
        assert!(avg_ask_price / 50.0 > 1.0 - PRECISION);
    }

    #[test]
    fn test_price_symmetry() {
        let orders = generate_limit_orders(1000000);
        let mut lesser: u64 = 0;
        let mut greater: u64 = 0;
        for order in &orders {
            if let OrderType::Limit { qty: _, price } = order.kind {
                let threshold = match order.side {
                    OrderSide::Ask => 150,
                    OrderSide::Bid => 50,
                };
                if price < threshold {
                    lesser += 1;
                } else {
                    greater += 1;
                }
            }
        }
        let ratio = lesser as f64 / greater as f64;
        assert!(ratio < 1.0 + PRECISION);
        assert!(ratio > 1.0 - PRECISION);
    }

    #[test]
    fn test_no_panic() {
        let mut order_gen = GaussianOrderGenerator::new(50.0, 1.0, 50.0, 1.0);
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(0);
        for i in 0..1_000_000 {
            order_gen.generate(
                0,
                4 * i + 0,
                (OrderSide::Bid, OrderType::Limit { qty: 0, price: 0 }),
                &mut seeded_rng,
            );
            order_gen.generate(
                0,
                4 * i + 1,
                (OrderSide::Ask, OrderType::Market { qty: 0 }),
                &mut seeded_rng,
            );
            order_gen.generate(
                0,
                4 * i + 2,
                (OrderSide::Bid, OrderType::Cancel { old_id: 0 }),
                &mut seeded_rng,
            );
            order_gen.generate(
                0,
                4 * i + 3,
                (
                    OrderSide::Ask,
                    OrderType::Update {
                        old_id: 0,
                        qty: 0,
                        price: 0,
                    },
                ),
                &mut seeded_rng,
            );
        }
    }
}
