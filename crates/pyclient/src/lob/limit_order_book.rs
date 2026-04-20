use mm_core::lob_core::{
    OrderId, OrderQty, Price,
    market_events::{L3Event, L3EventExtra, MarketEvent, MarketEventType, TradeEvent},
    market_orders::{LimitOrder, OrderSide, OrderType},
};
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
struct Level {
    pub qty: u64,
    pub order_count: u64,
}

/// A stripped down version of the OrderBook. Directly manages its state
/// via MarketEvents instead of handling matching logic, trade execution, etc.
#[derive(Default)]
pub struct OrderBook {
    user_orders: HashMap<OrderId, LimitOrder>,
    bid_levels: BTreeMap<Price, Level>,
    ask_levels: BTreeMap<Price, Level>,
}
impl OrderBook {
    /// Accepts a market event and updates the state of the book
    pub fn process_event(&mut self, event: MarketEvent) {
        match event.kind {
            MarketEventType::L3(e) => {
                match e.kind {
                    OrderType::Limit { qty, price } => self.handle_limit(e, qty, price),
                    OrderType::Update { old_id, qty, price } => {
                        self.handle_update(e, old_id, qty, price)
                    }
                    OrderType::Cancel { .. } => self.handle_cancel(e),
                    OrderType::Market { .. } => {
                        // Ignore market orders, the actual result of their execution is covered by the trade event
                    }
                }
            }
            MarketEventType::Trade(e) => self.handle_trade(e),
        }
    }

    fn handle_limit(&mut self, e: L3Event, qty: OrderQty, price: Price) {
        let level = match e.side {
            OrderSide::Ask => self.ask_levels.entry(price).or_default(),
            OrderSide::Bid => self.bid_levels.entry(price).or_default(),
        };
        level.qty += qty as u64;
        level.order_count += 1;
        self.user_orders.insert(
            e.order_id,
            LimitOrder {
                client_id: 0, // TODO: PLACEHOLDER, these aren't sent by MOLD, so maybe a stripped down struct should be used that doesn't have this
                order_id: e.order_id,
                side: e.side,
                status: mm_core::lob_core::market_orders::OrderStatus::Active,
                qty,
                price,
            },
        );
        // NOTE: LimitOrder events only denote limit orders being entered into the book, trades are executed via trade events
    }
    fn handle_cancel(&mut self, e: L3Event) {
        let (old_price, old_qty) = match self.user_orders.get_mut(&e.order_id) {
            Some(o) => (o.price, o.qty),
            None => {
                panic!(
                    "Expected to find order with id {} for cancel event, but it did not exist",
                    e.order_id
                );
            }
        };

        let side = match e.side {
            OrderSide::Ask => &mut self.ask_levels,
            OrderSide::Bid => &mut self.bid_levels,
        };
        let level_order_count = {
            let level = side.get_mut(&old_price).unwrap();
            level.qty -= old_qty as u64;
            level.order_count -= 1;
            self.user_orders.remove(&e.order_id);
            level.order_count
        };
        if level_order_count == 0 {
            side.remove(&old_price);
        }
    }
    fn handle_update(&mut self, e: L3Event, old_id: OrderId, qty: OrderQty, price: Price) {
        let old_order = match self.user_orders.get_mut(&old_id) {
            Some(o) => o,
            None => {
                panic!(
                    "Expected to find order with id {} for update event, but it did not exist",
                    old_id
                );
            }
        };

        let new_level = match e.side {
            OrderSide::Ask => self.ask_levels.entry(price).or_default(),
            OrderSide::Bid => self.bid_levels.entry(price).or_default(),
        };
        new_level.qty += qty as u64;
        new_level.order_count += 1;

        let old_side = match old_order.side {
            OrderSide::Ask => &mut self.ask_levels,
            OrderSide::Bid => &mut self.bid_levels,
        };
        // SAFETY: A valid update will always have an old order on the price level it is expected to be on
        let old_level = old_side.get_mut(&old_order.price).unwrap();
        old_level.qty -= old_order.qty as u64;
        old_level.order_count -= 1;
        if old_level.order_count == 0 {
            old_side.remove(&old_order.price);
        }

        self.user_orders.insert(
            e.order_id,
            LimitOrder {
                client_id: 0, // TODO: PLACEHOLDER, these aren't sent by MOLD, so maybe a stripped down struct should be used that doesn't have this
                order_id: e.order_id,
                side: e.side,
                status: mm_core::lob_core::market_orders::OrderStatus::Active,
                qty,
                price,
            },
        );
        self.user_orders.remove(&old_id);
    }
    fn handle_trade(&mut self, e: TradeEvent) {
        let maker = match self.user_orders.get_mut(&e.maker_id) {
            Some(o) => o,
            None => {
                panic!(
                    "Expected to find order with id {} for trade event, but it did not exist",
                    e.maker_id
                );
            }
        };
        //  NOTE: The market event includes the aggressor side, however it is currently broken
        //        MoldEngine does not properly parse/send the aggressor side, when read by MoldClient it always defaults to same value
        let side = match maker.side {
            //  SAFETY: A trade being made should always have an order that exists on the maker side at the given price level
            OrderSide::Ask => &mut self.ask_levels,
            OrderSide::Bid => &mut self.bid_levels,
        };
        let level_order_count = {
            let level = side.get_mut(&e.price).unwrap();
            level.qty -= e.quantity as u64;
            maker.qty -= e.quantity;
            if maker.qty == 0 {
                level.order_count -= 1;
                self.user_orders.remove(&e.maker_id);
            }
            level.order_count
        };
        if level_order_count == 0 {
            side.remove(&e.price);
        }
    }

    /// Returns the best bidding price or None if there are no bids
    pub fn best_bid(&self) -> Option<Price> {
        if let Some((price, _)) = self.bid_levels.last_key_value() {
            Some(*price)
        } else {
            None
        }
    }

    /// Returns a tuple of (best_bid_price,qty) or None if there are no bids
    pub fn best_bid_and_size(&self) -> Option<(Price, u64)> {
        if let Some((price, best)) = self.bid_levels.last_key_value() {
            Some((*price, best.qty))
        } else {
            None
        }
    }

    /// Returns the best asking price or None if there are no bids
    pub fn best_ask(&self) -> Option<Price> {
        if let Some((price, _)) = self.ask_levels.first_key_value() {
            Some(*price)
        } else {
            None
        }
    }

    /// Returns a tuple of (best_ask_price,qty) or None if there are no bids
    pub fn best_ask_and_size(&self) -> Option<(Price, u64)> {
        if let Some((price, best)) = self.ask_levels.first_key_value() {
            Some((*price, best.qty))
        } else {
            None
        }
    }

    /// Returns an average of the best asking and bidding prices or None if there are no bids or no orders
    pub fn mid_price(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            Some((best_ask as f64 + best_bid as f64) / 2.0)
        } else {
            None
        }
    }

    /// Returns the difference between the best asking and bidding prices or None if there are no bids or no orders
    pub fn spread(&self) -> Option<Price> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            Some(best_ask - best_bid)
        } else {
            None
        }
    }

    /// Returns the quantity of a given price level on the specified side
    pub fn get_level(&self, price: Price, side: OrderSide) -> u64 {
        match side {
            OrderSide::Ask => self.ask_levels.get(&price).map(|l| l.qty).unwrap_or(0),
            OrderSide::Bid => self.bid_levels.get(&price).map(|l| l.qty).unwrap_or(0),
        }
    }

    /// Returns the quantities of the top n price levels on the specified side
    pub fn get_top_levels(&self, side: OrderSide, n: usize) -> Vec<(Price, u64)> {
        match side {
            OrderSide::Ask => self
                .ask_levels
                .iter()
                .take(n)
                .map(|(p, l)| (*p, l.qty))
                .collect(),
            OrderSide::Bid => self
                .bid_levels
                .iter()
                .rev()
                .take(n)
                .map(|(p, l)| (*p, l.qty))
                .collect(),
        }
    }
}
