use std::collections::BTreeMap;
use std::collections::VecDeque;
use crate::lob::types::{OrderId, Price};
use crate::lob::order::Order;

#[derive(Debug, Default)]
pub struct PriceLevel {
	total_qty: u64,
	orders: VecDeque<OrderId>,
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
	orders: HashMap<OrderId, Order>,
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
}