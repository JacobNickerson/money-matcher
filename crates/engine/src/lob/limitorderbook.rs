use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::lob::order::{LimitOrder, Order, OrderSide, OrderStatus, OrderType};
use crate::lob::types::{OrderId, Price};

#[derive(Debug, Default)]
pub struct PriceLevel {
	pub total_qty: u64,
	pub orders: VecDeque<OrderId>,
}
impl PriceLevel {
	pub fn new() -> Self {
		Self {
			total_qty: 0,
			orders: VecDeque::new(),
		}
	}
	/// Drops order IDs from the front of the queue until reaching an ID where the associated order
	/// in the order book passed as an arg is active and has a non-zero quantity.
	/// 
	/// Returns the first order ID in the queue where the trade is active with a non-zero quantity
	/// Returns None if no active orders exist in queue with a non-zero quantity
	pub fn prune(&mut self, all_orders: &mut HashMap<OrderId, LimitOrder>) -> Option<OrderId> {
		while let Some(&order_id) = self.orders.front() {
			let order = &all_orders[&order_id];
			if order.status == OrderStatus::Active && order.qty > 0 {
				return Some(order_id);
			}
			self.orders.pop_front();
			self.total_qty -= order.qty;
		};
		None
	}
	/// Wrapper for pop_front()
	pub fn pop_front(&mut self) -> Option<OrderId> {
		self.orders.pop_front()
	}
	/// Wrapper for front()
	pub fn front(&self) -> Option<OrderId> {
		self.orders.front().copied()
	}
	/// Wrapper for push_back() that also updates total qty 
	pub fn push(&mut self, order: &LimitOrder) {
		self.orders.push_back(order.order_id);
		self.total_qty += order.qty;
	}
}

#[derive(Debug, Default)]
pub struct OrderBook {
	best_buy: Price,
	best_sell: Price,
	orders: HashMap<OrderId, LimitOrder>,
	buy_orders: BTreeMap<Price, PriceLevel>, 
	sell_orders: BTreeMap<Price, PriceLevel>,
	bid_orders: BTreeMap<Price, PriceLevel>, 
	ask_orders: BTreeMap<Price, PriceLevel>, 
}
impl OrderBook {
	pub fn new() -> Self {
		Self {
			orders: HashMap::new(),
			bid_orders: BTreeMap::new(),
			ask_orders: BTreeMap::new(),
		}
	}
	/// Accepts an Order and handles it according to its OrderType
	/// 
	/// LimitOrders are matched and added into LOB if not completely matched
	/// MarketOrders attempt to make qty trades starting from best price and partially fill
	///   if there is not enough liquidity
	/// CancelOrders attempt to cancel an order
	/// UpdateOrders cancel the previously existing order and resubmit a new order
	pub fn process_order(&mut self, order: Order) -> Option<LimitOrder> {
	// TODO: Update return type to be more informative
		match order.kind {
			OrderType::LimitOrder { qty, price } => {
				Some(self.add_order(
					LimitOrder::new(order, qty, price)
				))
			}
			OrderType::MarketOrder { mut qty } => {
				self.market_order(&mut qty, order.side);
				Some(LimitOrder::new(order, qty, 0))
			}
			OrderType::CancelOrder => {
				self.cancel_order(order.order_id)
			}
			OrderType::UpdateOrder { old_id, qty, price }=> {
				self.update_order(
					LimitOrder::new(order,qty,price),
					old_id
				)
			}
		}
	}
	/// Prunes lazily removed bid orders and returns the current best bid
	pub fn best_bid(&mut self) -> Option<Price> {
		// TODO: Will probably need to adjust this when perf-profiling and trying to reduce allocations
		let mut to_delete: Vec<Price> = vec![0; self.bid_orders.len()];
		let mut deleted_count: usize = 0;
		let mut best: Option<Price> = None;
		for (price, level) in self.bid_orders.iter_mut().rev() {
			level.prune(&mut self.orders);
			match level.front() {
				Some(_) => {
					best = Some(*price);
					break;
				},
				None => {
					to_delete[deleted_count] = *price;
					deleted_count += 1;
				}
			}
		};
		for i in 0..deleted_count {
			self.bid_orders.remove(
				&to_delete[i]	
			);
		} 
		best
	}
	/// Prunes lazily removed ask orders and returns the current best ask
	pub fn best_ask(&mut self) -> Option<Price> {
		let mut to_delete: Vec<Price> = vec![0; self.ask_orders.len()];
		let mut deleted_count: usize = 0;
		let mut best: Option<Price> = None;
		for (price, level) in self.ask_orders.iter_mut() {
			level.prune(&mut self.orders);
			match level.front() {
				Some(_) => {
					best = Some(*price);
					break;
				},
				None => {
					to_delete[deleted_count] = *price;
					deleted_count += 1;
				}
			}
		};
		for i in (0..deleted_count).rev() {
			self.ask_orders.remove(
				&to_delete[i]	
			);
		} 
		best
	}
	/// Executes a trade if a valid match can be made, see match_order() for details about matching.
	/// Adds an order to the side of the book specified in the order if any of the order's quantity is unmatched.
	fn add_order(&mut self, original_order: LimitOrder) -> LimitOrder {
		let mut order = original_order; 
		self.match_order(&mut order);
		if order.qty == 0 { return order; }
		let level = match order.side {
			OrderSide::Bid => {
				self.bid_orders
					.entry(order.price)
					.or_default()
			}
			OrderSide::Ask => {
				self.ask_orders
					.entry(order.price)
					.or_default()
			}
		};
		level.push(&order);
		self.orders.insert(order.order_id,order);
		order
	}
	/// Updates an existing order by cancelling it and replacing it with a new order. Executes
	/// a trade if a valid match can be made
	fn update_order(&mut self, order: LimitOrder, old_order_id: OrderId) -> Option<LimitOrder>{
		self
			.cancel_order(old_order_id)
			.map(|_| self.add_order(order))
	}
	/// Lazily cancels an order by marking it as canceled. Lazily canceled orders are pruned
	/// by `best_bid()`, `best_ask()`, or `match()`
	fn cancel_order(&mut self, order_id: OrderId) -> Option<LimitOrder> {
		match self.orders.get_mut(&order_id) {
			Some(order) => {
				order.status = OrderStatus::Canceled;
				Some(*order)
			},
			None => None
		}
	}
	/// Matches bid orders to ask orders with lower or equal prices.
	/// Matches ask orders to bid orders with higher or equal prices. 
	/// If a match is made, a trade is executed at the price of the order that already existed.
	fn match_order(&mut self, order: &mut LimitOrder) {
		match order.side {
			OrderSide::Ask => {
				// NOTE: Probably not necessary, but at least it prunes the book
				if self.best_bid().unwrap_or(0) < order.price {
					return;
				}
				for (price, level) in self.bid_orders.iter_mut().rev() {
					if order.qty == 0 || *price < order.price { break; }	
					while let Some(buy_order_id) = level.front() {
						// NOTE: Can panic, but an id in a price level should always be in orders until it is pruned
						let mut buy_order = self.orders[&buy_order_id];
						let trade_volume = std::cmp::min(buy_order.qty,order.qty);
						buy_order.qty -= trade_volume;
						order.qty -= trade_volume;
						if buy_order.qty == 0 { level.pop_front(); }
						else { break; }
						// TODO: IMPLEMENT TRADE EVENT EMITTING
					}
				}
			}
			OrderSide::Bid => {
				// NOTE: This might be janky, maybe implement some additional verification of order prices to avoid having this cause issues
				if self.best_ask().unwrap_or(u64::MAX) > order.price {
					return;
				}
				for (price, level) in self.ask_orders.iter_mut() {
					if order.qty == 0 || *price > order.price { break; }	
					while let Some(sell_order_id) = level.front() {
						// NOTE: Can panic, but an id in a price level should always be in orders until it is pruned
						let mut sell_order = self.orders[&sell_order_id];
						let trade_volume = std::cmp::min(sell_order.qty,order.qty);
						sell_order.qty -= trade_volume;
						order.qty -= trade_volume;
						if sell_order.qty == 0 { level.pop_front(); }
						else { break; }
						// TODO: IMPLEMENT TRADE EVENT EMITTING
					}
				}
			}
		};
	}

	fn market_order(&mut self, qty: &mut u64, side: OrderSide) {
		match side {
			OrderSide::Ask => {
				for (price, level) in self.bid_orders.iter_mut().rev() {
					while let Some(buy_order_id) = level.front() {
						// NOTE: Can panic, but an id in a price level should always be in orders until it is pruned
						let mut buy_order = self.orders[&buy_order_id];
						let trade_volume = std::cmp::min(buy_order.qty,*qty);
						buy_order.qty -= trade_volume;
						*qty -= trade_volume;
						if buy_order.qty == 0 { level.pop_front(); }
						else { break; }
						// TODO: IMPLEMENT TRADE EVENT EMITTING
					}
				}
			}
			OrderSide::Bid => {
				// NOTE: This might be janky, maybe implement some additional verification of order prices to avoid having this cause issues
				for (price, level) in self.ask_orders.iter_mut() {
					while let Some(sell_order_id) = level.front() {
						// NOTE: Can panic, but an id in a price level should always be in orders until it is pruned
						let mut sell_order = self.orders[&sell_order_id];
						let trade_volume = std::cmp::min(sell_order.qty,*qty);
						sell_order.qty -= trade_volume;
						*qty -= trade_volume;
						if sell_order.qty == 0 { level.pop_front(); }
						else { break; }
						// TODO: IMPLEMENT TRADE EVENT EMITTING
					}
				}
			}
		};
	}
}

/* UNIT TESTS */
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
    fn empty_book_has_no_best_prices() {
        let mut book = OrderBook::new();
        assert!(book.best_bid().is_none());
        assert!(book.best_ask().is_none());
    }

	#[test]
	fn add_bid_without_crossing() {
		let mut book = OrderBook::new();
		book.process_order(
			Order::new(0,OrderSide::Bid,1,OrderType::LimitOrder { qty: 1, price: 100 })
		);
		book.process_order(
			Order::new(0,OrderSide::Ask,1,OrderType::LimitOrder { qty: 1, price: 200 })
		);
		assert_eq!(book.best_bid(), Some(100));
		assert_eq!(book.best_ask(), Some(200));
	}

	#[test]
	fn cancel_removes_order() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Bid,1,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		assert!(
			book
				.process_order(Order::new(0,OrderSide::Bid,1,OrderType::CancelOrder))
				.is_some()
		);
		assert!(book.best_bid().is_none());
	}

	#[test]
	fn pruning_multiple_price_levels() {
		let mut book = OrderBook::new();

		for i in 0..=2 {
			book.process_order(
				Order::new(i,OrderSide::Bid,i,OrderType::LimitOrder { qty: 5, price: 100 + 5*i })
			);
		}
		assert_eq!(book.best_bid(), Some(110));
		assert!(book.cancel_order(1).is_some());
		assert!(book.cancel_order(2).is_some());
		assert_eq!(book.best_bid(), Some(100));
	}

	#[test]
	fn cancel_nonexistent_returns_none() {
		let mut book = OrderBook::new();
		assert!(
			book
				.process_order(Order::new(0,OrderSide::Bid,1,OrderType::CancelOrder))
				.is_none()
		);
	}
	
	#[test]
	fn update_order_updates_order() {
		let mut book = OrderBook::new();
		book.process_order(
			Order::new(0,OrderSide::Bid,1,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		assert_eq!(book.best_bid(), Some(100));
		book.process_order(
			Order::new(1,OrderSide::Bid,1,OrderType::UpdateOrder { old_id: 0, qty: 5, price: 500 })
		);
		assert_eq!(book.best_bid(), Some(500));
	}

	#[test]
	fn update_nonexistent_order_has_no_effect() {
		let mut book = OrderBook::new();
		book.process_order(
			Order::new(0,OrderSide::Bid,1,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		assert_eq!(book.best_bid(), Some(100));
		book.process_order(
			Order::new(1,OrderSide::Bid,1,OrderType::UpdateOrder { old_id: 1, qty: 5, price: 500 })
		);
		assert_eq!(book.best_bid(), Some(100));
	}

	#[test]
	fn best_bid_is_highest_price() {
		let mut book = OrderBook::new();

		for i in 0..=2 {
			book.process_order(
				Order::new(i,OrderSide::Bid,i,OrderType::LimitOrder { qty: 5, price: 100 + 5*i })
			);
		}
		assert_eq!(book.best_bid(), Some(110));
	}

	#[test]
	fn many_orders_do_not_panic() {
		let mut book = OrderBook::new();

		for i in 0..1_000_000 {
			book.process_order(
				Order::new(i,OrderSide::Bid,i,OrderType::LimitOrder { qty: 10, price: 100 + (i%10) })
			);
		}

		assert!(book.best_bid().is_some());
	}

	#[test]
	fn fifo_within_price_level() {
		let mut book = OrderBook::new();

		for i in 0..2 {
			book.process_order(
				Order::new(i,OrderSide::Ask,i,OrderType::LimitOrder { qty: 5, price: 100 })
			);
		}

		book.process_order(
			Order::new(2,OrderSide::Bid,2,OrderType::LimitOrder { qty: 6, price: 100 })
		);

		// TODO: Determine structure for trades and test it here
		// TODO: Test that order 0 is made first

		assert_eq!(book.best_bid(), None);
		assert_eq!(book.best_ask(), Some(100));
	}

	#[test]
	fn simple_full_match() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		book.process_order(
			Order::new(1,OrderSide::Ask,1,OrderType::LimitOrder { qty: 5, price: 100 })
		);

		// TODO: Determine structure for trades and test it here

		assert!(book.best_bid().is_none());
		assert!(book.best_ask().is_none());
	}

	#[test]
	fn partial_match_leaves_resting_qty() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		book.process_order(
			Order::new(1,OrderSide::Ask,1,OrderType::LimitOrder { qty: 6, price: 100 })
		);

		// TODO: Determine structure for trades and test it here

		assert_eq!(book.best_ask(), Some(100));
	}

	#[test]
	fn multi_level_sweep() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		book.process_order(
			Order::new(1,OrderSide::Ask,1,OrderType::LimitOrder { qty: 5, price: 105 })
		);
		book.process_order(
			Order::new(2,OrderSide::Bid,2,OrderType::LimitOrder { qty: 6, price: 105 })
		);

		// TODO: Determine structure for trades and test it here
		assert_eq!(book.best_ask(), Some(105));
	}

	#[test]
	fn market_order_single_level() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		assert_eq!(book.best_ask(), Some(100));
		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::MarketOrder { qty: 5 })
		);
		// TODO: Determine structure for trades and test it here
		assert!(book.best_ask().is_none());
	}

	#[test]
	fn market_order_multi_level() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 150 })
		);
		assert_eq!(book.best_ask(), Some(100));
		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::MarketOrder { qty: 10 })
		);
		// TODO: Determine structure for trades and test it here
		assert!(book.best_ask().is_none());
	}

	#[test]
	fn market_order_partial_fill() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 100 })
		);
		book.process_order(
			Order::new(0,OrderSide::Ask,0,OrderType::LimitOrder { qty: 5, price: 150 })
		);
		assert_eq!(book.best_ask(), Some(100));
		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::MarketOrder { qty: 15 })
		);
		// TODO: Determine structure for trades and test it here
		assert!(book.best_ask().is_none());
	}

	#[test]
	fn market_order_no_fill() {
		let mut book = OrderBook::new();

		book.process_order(
			Order::new(0,OrderSide::Bid,0,OrderType::MarketOrder { qty: 15 })
		);
		// TODO: Determine structure for trades and test it here
		assert!(book.best_ask().is_none());
	}
}