use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::lob::types::Side;
use crate::lob::types::{OrderId, Price};
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
}

#[derive(Debug, Default)]
pub struct OrderBook {
	best_buy: Price,
	best_sell: Price,
	orders: HashMap<OrderId, LimitOrder>,
	buy_orders: BTreeMap<Price, PriceLevel>, 
	sell_orders: BTreeMap<Price, PriceLevel>, 
}
impl OrderBook {
	pub fn new() -> Self {
		Self {
			best_buy: 0,
			best_sell: 0,
			orders: HashMap::new(),
			buy_orders: BTreeMap::new(),
			sell_orders: BTreeMap::new(),
		}
	}
	pub fn add_order(&mut self, order: LimitOrder) -> LimitOrder {
		let level = match order.side {
			Side::Buy => {
				self.best_buy = std::cmp::max(self.best_buy,order.price);
				self.buy_orders
					.entry(order.price)
					.or_insert_with(PriceLevel::new)
			}
			Side::Sell => {
				self.best_sell = std::cmp::min(self.best_sell,order.price);
				self.sell_orders
					.entry(order.price)
					.or_insert_with(PriceLevel::new)
			}
		};
		level.total_qty += order.qty;
		order
	}
	pub fn update_order(&mut self, order: LimitOrder) -> Option<LimitOrder>{
		self.orders
			.get_mut(&order.order_id)
			.map(|old_order| {
				*old_order = order;
				order
			})
	}
	pub fn delete_order(&mut self, order_id: OrderId) -> Option<LimitOrder> {
		self.orders
			.remove(&order_id)
	}
}