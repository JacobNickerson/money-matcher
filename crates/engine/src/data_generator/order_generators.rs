use circular_buffer::CircularBuffer;
use fastrand;
use mm_core::lob_core::{
    ClientId, OrderId, OrderQty, Price, Timestamp,
    market_orders::{Order, OrderSide, OrderType},
};
use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::simulator::simulator::SimTime;

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

const QUANTITIES: [OrderQty; 5] = [1, 2, 5, 10, 20];

pub struct GaussianOrderGenerator {
    dist: Normal<f64>,
    current_time: SimTime,
    order_counter: u64,
}
impl GaussianOrderGenerator {
    const ACTIVE_ORDER_BUFFER_SIZE: usize = 1_000_000;
    pub fn new(mean: f64, deviation: f64) -> Self {
        Self {
            dist: Normal::new(mean, deviation).unwrap(),
            current_time: 0,
            order_counter: 0,
        }
    }
    fn compute_price(&mut self, rng: &mut impl Rng) -> Price {
        (self.dist.sample(rng).abs() * 100.0) as Price
    }
    fn get_active_order(&self, side: OrderSide) -> OrderId {
        fastrand::u64(0..self.order_counter)
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
        let price = self.compute_price(rng);
        let qty = QUANTITIES[fastrand::usize(0..QUANTITIES.len())];
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
                OrderType::Cancel { old_id: self.get_active_order(side) },
            ),
            OrderType::Update {
                ..
            } => Order::new(
                client_id,
                0, // NOTE: Use a junk value, simulator sets this on receipt
                side,
                self.current_time,
                OrderType::Update {
                    old_id: self.get_active_order(side),
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
        let mut order_gen = GaussianOrderGenerator::new(50.0, 1.0);
        let mut seeded_rng = ChaCha8Rng::seed_from_u64(0);
        for i in 0..count {
            orders.push(order_gen.generate(
                0,
                i,
                (OrderSide::Bid, OrderType::Limit { qty: 0, price: 0 }),
                &mut seeded_rng,
            ));
        }
        orders
    }

    #[test]
    fn test_price_mean() {
        let orders = generate_limit_orders(1000000);
        let mut total_price: u64 = 0;
        for order in &orders {
            if let OrderType::Limit { qty: _, price } = order.kind {
                total_price += price as u64;
            }
        }
        let avg_price = total_price as f64 / 1000000.0;
        assert!(avg_price / 5000.0 < 1.0 + PRECISION);
        assert!(avg_price / 5000.0 > 1.0 - PRECISION);
    }

    #[test]
    fn test_price_symmetry() {
        let orders = generate_limit_orders(1000000);
        let mut lesser: u64 = 0;
        let mut greater: u64 = 0;
        for order in &orders {
            if let OrderType::Limit { qty: _, price } = order.kind {
                if price < 5000 {
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
        let mut order_gen = GaussianOrderGenerator::new(50.0, 1.0);
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
