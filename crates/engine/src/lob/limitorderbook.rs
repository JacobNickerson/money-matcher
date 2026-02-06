use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::lob::types::Side;
use crate::lob::types::{OrderId, OrderStatus, Price};
use crate::lob::limitorder::LimitOrder;

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
	pub fn add_order(&mut self, original_order: LimitOrder) -> LimitOrder {
		let mut order = original_order; 
		self.match_order(&mut order);
		if order.qty == 0 { return order; }
		let level = match order.side {
			Side::Bid => {
				self.bid_orders
					.entry(order.price)
					.or_default()
			}
			Side::Ask => {
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
	pub fn update_order(&mut self, order: LimitOrder, old_order_id: OrderId) -> Option<LimitOrder>{
		self
			.cancel_order(old_order_id)
			.map(|_| self.add_order(order))
	}
	/// Lazily cancels an order by marking it as canceled. Lazily canceled orders are pruned
	/// by `best_bid()`, `best_ask()`, or `match()`
	pub fn cancel_order(&mut self, order_id: OrderId) -> Option<LimitOrder> {
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
			Side::Ask => {
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
			Side::Bid => {
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
		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Ask, 200, 5, 1)
		);
		assert_eq!(book.best_bid(), Some(100));
		assert_eq!(book.best_ask(), Some(200));
	}

	#[test]
	fn cancel_removes_order() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		assert!(book.cancel_order(0).is_some());
		assert!(book.best_bid().is_none());
	}

	#[test]
	fn pruning_multiple_price_levels() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Bid, 105, 5, 1)
		);
		book.add_order(
			LimitOrder::new(2, Side::Bid, 110, 5, 2)
		);
		assert_eq!(book.best_bid(), Some(110));
		assert!(book.cancel_order(1).is_some());
		assert!(book.cancel_order(2).is_some());
		assert_eq!(book.best_bid(), Some(100));
	}

	#[test]
	fn cancel_nonexistent_returns_none() {
		let mut book = OrderBook::new();
		assert!(book.cancel_order(42).is_none());
	}
	
	#[test]
	fn update_order_updates_order() {
		let mut book = OrderBook::new();
		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		assert_eq!(book.best_bid(), Some(100));
		book.update_order(
			LimitOrder::new(0, Side::Bid, 500, 5, 0),
			0
		);
		assert_eq!(book.best_bid(), Some(500));
	}

	#[test]
	fn update_nonexistent_order_has_no_effect() {
		let mut book = OrderBook::new();
		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		assert_eq!(book.best_bid(), Some(100));
		book.update_order(
			LimitOrder::new(10, Side::Bid, 500, 5, 0),
			10
		);
		assert_eq!(book.best_bid(), Some(100));
	}

	#[test]
	fn best_bid_is_highest_price() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Bid, 105, 5, 1)
		);
		book.add_order(
			LimitOrder::new(2, Side::Bid, 110, 5, 2)
		);

		assert_eq!(book.best_bid(), Some(110));
	}

	#[test]
	fn many_orders_do_not_panic() {
		let mut book = OrderBook::new();

		for i in 0..1_000_000 {
			book.add_order(
				LimitOrder::new(i, Side::Bid, 100 + (i%10), 5, i)
			);
		}

		assert!(book.best_bid().is_some());
	}

	#[test]
	fn fifo_within_price_level() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Ask, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Ask, 100, 5, 1)
		);

		book.add_order(
			LimitOrder::new(2, Side::Bid, 100, 6, 2)
		);

		// TODO: Determine structure for trades and test it here
		// TODO: Test that order 0 is made first

		assert_eq!(book.best_bid(), None);
		assert_eq!(book.best_ask(), Some(100));
	}

	#[test]
	fn simple_full_match() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Ask, 100, 5, 1)
		);

		// TODO: Determine structure for trades and test it here

		assert!(book.best_bid().is_none());
		assert!(book.best_ask().is_none());
	}

	#[test]
	fn partial_match_leaves_resting_qty() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Bid, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(0, Side::Ask, 100, 10, 1)
		);

		// TODO: Determine structure for trades and test it here

		assert_eq!(book.best_ask(), Some(100));
	}

	#[test]
	fn multi_level_sweep() {
		let mut book = OrderBook::new();

		book.add_order(
			LimitOrder::new(0, Side::Ask, 100, 5, 0)
		);
		book.add_order(
			LimitOrder::new(1, Side::Ask, 105, 5, 1)
		);
		book.add_order(
			LimitOrder::new(2, Side::Bid, 105, 8, 2)
		);

		// TODO: Determine structure for trades and test it here
		assert_eq!(book.best_ask(), Some(105));
	}
}