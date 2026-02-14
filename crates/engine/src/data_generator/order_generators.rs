use rand::{Rng};
use rand_distr::{Distribution, Normal};
use circular_buffer::CircularBuffer;
use fastrand;
use crate::lob::types::{OrderId, Price};
use crate::lob::order::{Order, OrderSide, OrderType};

/// Generates the next event, also handles selection of price level, memory of orders for cancellation
pub trait OrderGenerator {
	/// Accepts an order type and randomly samples some distribution to generate an Order
	fn generate(&mut self, time_stamp: u64, order_variant: (OrderSide, OrderType), rng: &mut impl Rng) -> Order;
}

const QUANTITIES: [u64; 5] = [1,2,5,10,20];

pub struct GaussianOrderGenerator {
	dist: Normal<f64>,
	order_counter: u64,
	active_bids: Box<CircularBuffer<{ Self::ACTIVE_ORDER_BUFFER_SIZE },OrderId>>,
	active_asks: Box<CircularBuffer<{ Self::ACTIVE_ORDER_BUFFER_SIZE },OrderId>>,
}
impl GaussianOrderGenerator {
	const ACTIVE_ORDER_BUFFER_SIZE: usize = 1_000_000;
	pub fn new(mean: f64, deviation: f64) -> Self {
		let mut this = Self {
			dist: Normal::new(mean,deviation).unwrap(),
			order_counter: 0,
			active_bids: CircularBuffer::boxed(),
			active_asks: CircularBuffer::boxed()
		};
		// zero memory to prevent panics if a cancel is made before this is fully saturated
		for _ in 0..Self::ACTIVE_ORDER_BUFFER_SIZE {
			this.active_asks.push_back(0);
			this.active_bids.push_back(0);
		}
		this
	}
	fn compute_price(&mut self, rng: &mut impl Rng) -> Price {
		(self.dist.sample(rng).abs() * 100.0) as Price
	}
	fn get_active_order(&self, side: OrderSide) -> OrderId {
		let ind = fastrand::usize(0..Self::ACTIVE_ORDER_BUFFER_SIZE);
		match side {
			OrderSide::Ask => { self.active_asks[ind] },
			OrderSide::Bid => { self.active_bids[ind] },
		}
	}
}
impl OrderGenerator for GaussianOrderGenerator {
	fn generate(&mut self, time_stamp: u64, order_variant: (OrderSide, OrderType), rng: &mut impl Rng) -> Order {
		let (side, kind) = order_variant;
		let price = self.compute_price(rng);
		let qty = QUANTITIES[fastrand::usize(0..QUANTITIES.len())]; 
		self.order_counter += 1;
		match kind {
			OrderType::LimitOrder { qty: _, price: _ } => {
				match side {
					OrderSide::Ask => { self.active_asks.push_back(self.order_counter) },
					OrderSide::Bid => { self.active_bids.push_back(self.order_counter) },
				};
				Order::new(
					self.order_counter,
					side,
					time_stamp,
					OrderType::LimitOrder { qty, price }
				)
			}
			OrderType::MarketOrder { qty: _ } => {
				Order::new(
					self.order_counter,
					side,
					time_stamp,
					OrderType::MarketOrder { qty }
				)
			}
			OrderType::CancelOrder => {
				Order::new(
					self.get_active_order(side),
					side,
					time_stamp,
					OrderType::CancelOrder
				)
			}
			OrderType::UpdateOrder { old_id: _, qty: _, price: _ } => {
				Order::new(
					self.order_counter,
					side,
					time_stamp,
					OrderType::UpdateOrder { old_id: self.get_active_order(side), qty, price }
				)
			}
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
		let mut order_gen = GaussianOrderGenerator::new(50.0,1.0);
		let mut seeded_rng = ChaCha8Rng::seed_from_u64(0);
		for i in 0..count {
			orders.push(order_gen.generate(i, (OrderSide::Bid, OrderType::LimitOrder { qty: 0, price: 0 }), &mut seeded_rng));
		}
		orders
	}

	#[test]
	fn test_price_mean() {
		let orders = generate_limit_orders(1000000);	
		let mut total_price: Price = 0;
		for order in &orders {
			if let OrderType::LimitOrder { qty: _, price } = order.kind {
   					total_price += price;
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
			if let OrderType::LimitOrder { qty: _, price } = order.kind {
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
		let mut order_gen = GaussianOrderGenerator::new(50.0,1.0);
		let mut seeded_rng = ChaCha8Rng::seed_from_u64(0);
		for i in 0..1_000_000 {
			order_gen.generate(4*i+0, (OrderSide::Bid, OrderType::LimitOrder { qty: 0, price: 0 }), &mut seeded_rng);
			order_gen.generate(4*i+1, (OrderSide::Ask, OrderType::MarketOrder { qty: 0 }), &mut seeded_rng);
			order_gen.generate(4*i+2, (OrderSide::Bid, OrderType::CancelOrder), &mut seeded_rng);
			order_gen.generate(4*i+3, (OrderSide::Ask, OrderType::UpdateOrder { old_id: 0, qty: 0, price: 0 }), &mut seeded_rng);
		}
	}
}