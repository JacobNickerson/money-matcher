use mm_core::lob_core::{
    OrderId, OrderQty, Price, Timestamp,
    market_events::{
        ClientEvent, ClientEventType, EventSink, L3Event, LiquidityFlag, MarketEvent,
        MarketEventType, TradeEvent,
    },
    market_orders::{LimitOrder, Order, OrderSide, OrderStatus, OrderType},
};
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Struct representing a single price level within a limit order book. Contains a FIFO queue for order IDs, which should
/// be used to look up orders from a table
#[derive(Debug, Default)]
pub struct PriceLevel {
    pub total_qty: u64,
    orders: VecDeque<OrderId>,
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
    /// Updates cached values
    ///
    /// Returns the first order ID in the queue where the trade is active with a non-zero quantity
    /// Returns None if no active orders exist in queue with a non-zero quantity
    pub fn prune(&mut self, all_orders: &mut HashMap<OrderId, LimitOrder>) -> Option<OrderId> {
        while let Some(&order_id) = self.orders.front() {
            if let std::collections::hash_map::Entry::Occupied(entry) = all_orders.entry(order_id) {
                let order = *entry.get();
                if order.status == OrderStatus::Active && order.qty > 0 {
                    return Some(order_id);
                } else {
                    self.orders.pop_front();
                    self.total_qty -= order.qty as u64;
                    entry.remove();
                }
            }
        }
        None
    }

    /// Wrapper for pop_front()
    /// Does not update cached values
    pub fn pop_front(&mut self) -> Option<OrderId> {
        self.orders.pop_front()
    }

    /// Wrapper for front()
    pub fn front(&self) -> Option<OrderId> {
        self.orders.front().copied()
    }

    /// Wrapper for push_back() that also updates total qty
    /// Updates cached values
    pub fn push(&mut self, order: &LimitOrder) {
        self.orders.push_back(order.order_id);
        self.total_qty += order.qty as u64;
    }
}

#[derive(Debug)]
/// Struct representing a limit order book, stores all unmatched bids and asks and
/// On market order or attempting to push a limit order, attempts to match and execute viable trades
/// Emits L1/L2/L3/Trade/Client market events through push() provided by the event_sink passed on construction
pub struct OrderBook<T: EventSink> {
    orders: HashMap<OrderId, LimitOrder>,
    bid_orders: BTreeMap<Price, PriceLevel>,
    ask_orders: BTreeMap<Price, PriceLevel>,
    event_sink: T,
    best_bid: Price, // NOTE: These are updated after every call to process_order(), however within that function
    best_ask: Price, //       they should be considered potentially out of date
    total_asks: u64,
    total_bids: u64,
    market_event_counter: u16,
    client_event_counter: u64,
}
impl<T: EventSink> OrderBook<T> {
    pub fn new(event_sink: T) -> Self {
        Self {
            orders: HashMap::new(),
            bid_orders: BTreeMap::new(),
            ask_orders: BTreeMap::new(),
            event_sink,
            best_bid: 0,
            best_ask: 0,
            total_asks: 0,
            total_bids: 0,
            market_event_counter: 0,
            client_event_counter: 0,
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
        let time = order.timestamp;
        let order: Option<LimitOrder> = match order.kind {
            OrderType::Limit { .. } => self.add_order_and_emit_events(order, time),
            OrderType::Market { .. } => self.execute_market_order_and_emit_events(order, time),
            OrderType::Cancel { old_id } => self.cancel_order_and_emit_events(old_id, order, time),
            OrderType::Update {
                old_id,
                qty: _,
                price: _,
            } => self.update_order_and_emit_events(old_id, order, time),
        };
        self.update_aggregates();
        order
    }

    /// Prunes lazily removed bid orders and returns the current best bid
    /// Does not update the cached value of best bid
    pub fn best_bid(&mut self) -> Option<Price> {
        let mut to_delete: Vec<Price> = vec![0; self.bid_orders.len()];
        let mut deleted_count: usize = 0;
        let mut best: Option<Price> = None;
        for (price, level) in self.bid_orders.iter_mut().rev() {
            level.prune(&mut self.orders);
            match level.front() {
                Some(_) => {
                    best = Some(*price);
                    break;
                }
                None => {
                    to_delete[deleted_count] = *price;
                    deleted_count += 1;
                }
            }
        }
        for ind in to_delete.iter().take(deleted_count) {
            self.bid_orders.remove(ind);
        }
        best
    }

    /// Prunes lazily removed ask orders and returns the current best ask
    /// Does not update the cached value of best ask
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
                }
                None => {
                    to_delete[deleted_count] = *price;
                    deleted_count += 1;
                }
            }
        }
        for ind in to_delete.iter().take(deleted_count) {
            self.ask_orders.remove(ind);
        }
        best
    }

    /// Executes a trade if a valid match can be made, see match_order() for details about matching.
    /// Adds an order to the side of the book specified in the order if any of the order's quantity is unmatched.
    /// Possibly emits MarketEvents
    fn add_order_and_emit_events(
        &mut self,
        original_order: Order,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        let mut order = LimitOrder::new(original_order);
        if order.qty == 0 {
            self.reject_order(original_order, time);
            return None;
        }
        self.accept_order(original_order, time, order.qty);
        self.match_order(&mut order, time);
        if order.qty == 0 {
            return Some(order);
        }

        self.event_sink.push_event(MarketEvent::new(
            self.market_event_counter,
            time,
            MarketEventType::L3(L3Event::new_limit(order, original_order.timestamp)),
        ));
        let _ = self.market_event_counter.wrapping_add(1u16);
        let level = match order.side {
            OrderSide::Bid => {
                self.total_bids += order.qty as u64;
                self.bid_orders.entry(order.price).or_default()
            }
            OrderSide::Ask => {
                self.total_asks += order.qty as u64;
                self.ask_orders.entry(order.price).or_default()
            }
        };
        level.push(&order);
        self.orders.insert(order.order_id, order);
        Some(order)
    }

    /// Updates an existing order by cancelling it and replacing it with a new order. Executes
    /// a trade if a valid match can be made
    ///
    /// Emits ClientEvents for the cancellation, the new order, any trades that are made, and acknowledgement of the update
    fn update_order_and_emit_events(
        &mut self,
        old_id: OrderId,
        order: Order,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        let old_order = match self.orders.get_mut(&old_id) {
            Some(old_order) => old_order,
            None => {
                self.reject_order(order, time);
                return None;
            }
        };
        if old_order.status == OrderStatus::Canceled || old_order.qty == 0 {
            self.reject_order(order, time);
            return None;
        }

        self.event_sink.push_event(MarketEvent::new_update(
            self.market_event_counter,
            time,
            order,
        ));
        let _ = self.market_event_counter.wrapping_add(1u16);
        self.event_sink.push_client_event(ClientEvent {
            client_id: order.client_id,
            id: self.client_event_counter,
            timestamp: time,
            order_id: order.order_id,
            order_side: order.side,
            kind: ClientEventType::Updated,
            liquidity_flag: LiquidityFlag::Invalid,
        });
        self.client_event_counter += 1;

        // Cancelling the previous
        let level = match old_order.side {
            OrderSide::Ask => {
                self.total_asks -= old_order.qty as u64;
                self.ask_orders.get_mut(&old_order.price).unwrap() // If a valid old order is passed, then the price level should always exist
            }
            OrderSide::Bid => {
                self.total_bids -= old_order.qty as u64;
                self.bid_orders.get_mut(&old_order.price).unwrap() // If a valid old order is passed, then the price level should always exist
            }
        };
        level.total_qty -= old_order.qty as u64;
        old_order.qty = 0;
        old_order.status = OrderStatus::Canceled;

        // Adding the new
        let mut order: LimitOrder = LimitOrder::new(order);
        self.match_order(&mut order, time);
        if order.qty == 0 {
            return Some(order);
        }
        let level = match order.side {
            OrderSide::Bid => {
                self.total_bids += order.qty as u64;
                self.bid_orders.entry(order.price).or_default()
            }
            OrderSide::Ask => {
                self.total_asks += order.qty as u64;
                self.ask_orders.entry(order.price).or_default()
            }
        };
        level.push(&order);
        self.orders.insert(order.order_id, order);
        Some(order)
    }

    /// Lazily cancels an order by marking it as canceled. Lazily canceled orders are pruned
    /// by `best_bid()`, `best_ask()`, or `match_order()`
    ///
    /// Assumes that the old_order passed is a valid limit order from the book, but will do additional checks,
    /// like if the order is already canceled or has a quantity of 0, before emitting events
    ///
    /// Emits an invalid client event if the order is already canceled or has a quantity of 0, otherwise emits
    /// a cancel market and client event
    fn cancel_order_and_emit_events(
        &mut self,
        old_id: OrderId,
        order: Order,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        // TODO: old_id is a CLIENT order ID, need to resolve to engine order_id
        let old_order = match self.orders.get_mut(&old_id) {
            Some(old_order) => old_order,
            None => {
                self.reject_order(order, time);
                return None;
            }
        };
        if old_order.status == OrderStatus::Canceled || old_order.qty == 0 {
            self.reject_order(order, time);
            return None;
        }

        self.event_sink.push_event(MarketEvent::new_cancel(
            self.market_event_counter,
            time,
            order,
            old_order.qty,
        ));
        let _ = self.market_event_counter.wrapping_add(1u16);
        self.event_sink.push_client_event(ClientEvent {
            client_id: order.client_id,
            id: self.client_event_counter,
            timestamp: time,
            order_id: order.order_id,
            order_side: order.side,
            kind: ClientEventType::Canceled,
            liquidity_flag: LiquidityFlag::Invalid,
        });
        self.client_event_counter += 1;

        let level = match old_order.side {
            OrderSide::Ask => {
                self.total_asks -= old_order.qty as u64;
                self.ask_orders.get_mut(&old_order.price).unwrap() // If a valid old order is passed, then the price level should always exist
            }
            OrderSide::Bid => {
                self.total_bids -= old_order.qty as u64;
                self.bid_orders.get_mut(&old_order.price).unwrap() // If a valid old order is passed, then the price level should always exist
            }
        };

        level.total_qty -= old_order.qty as u64;
        old_order.qty = 0;
        old_order.status = OrderStatus::Canceled;

        Some(*old_order)
    }

    /// Matches bid orders to ask orders with lower or equal prices.
    /// Matches ask orders to bid orders with higher or equal prices.
    /// If a match is made, a trade is executed at the price of the order that already existed.
    /// Everytime a trade is made, one trade event and two client events are emitted
    fn match_order(&mut self, order: &mut LimitOrder, time: Timestamp) {
        match order.side {
            OrderSide::Bid => Self::make_trades(
                self.ask_orders.iter_mut(),
                &mut self.orders,
                &mut self.event_sink,
                order,
                time,
                &mut self.market_event_counter,
                &mut self.client_event_counter,
            ),
            OrderSide::Ask => Self::make_trades(
                self.bid_orders.iter_mut().rev(),
                &mut self.orders,
                &mut self.event_sink,
                order,
                time,
                &mut self.market_event_counter,
                &mut self.client_event_counter,
            ),
        }
    }

    /// Accepts an iterator in either direction across a BTreeMap mapping prices to their price levels
    /// Repeatedly makes trades until no more matches can be made
    /// Each trade emits a trade event and two client events
    fn make_trades<'a, 'b>(
        iter: impl Iterator<Item = (&'a Price, &'b mut PriceLevel)>,
        orders: &mut HashMap<OrderId, LimitOrder>,
        event_sink: &mut T,
        taker: &mut LimitOrder,
        time: Timestamp,
        market_event_counter: &mut u16,
        client_event_counter: &mut u64,
    ) {
        for (price, level) in iter {
            match taker.side {
                OrderSide::Ask => {
                    if *price < taker.price {
                        break;
                    }
                }
                OrderSide::Bid => {
                    if *price > taker.price {
                        break;
                    }
                }
            }
            while taker.qty > 0
                && let Some(maker_id) = level.front()
            {
                // NOTE: Can panic, but an id in a price level should always be in orders until it is pruned
                let maker = orders.get_mut(&maker_id).unwrap();
                if maker.qty == 0 || maker.status == OrderStatus::Canceled {
                    level.pop_front();
                    continue;
                }
                let trade_volume: OrderQty = std::cmp::min(maker.qty, taker.qty);
                maker.qty -= trade_volume;
                taker.qty -= trade_volume;

                event_sink.push_event(MarketEvent::new(
                    *market_event_counter,
                    time,
                    MarketEventType::Trade(TradeEvent {
                        price: *price,
                        quantity: trade_volume,
                        aggressor_side: taker.side,
                        maker_id: maker.order_id,
                    }),
                ));
                let _ = market_event_counter.wrapping_add(1u16);

                event_sink.push_client_event(ClientEvent {
                    client_id: taker.client_id,
                    id: *client_event_counter,
                    timestamp: time,
                    order_id: taker.order_id,
                    order_side: taker.side,
                    kind: match taker.qty == 0 {
                        true => ClientEventType::Filled,
                        false => ClientEventType::PartiallyFilled(taker.qty),
                    },
                    liquidity_flag: LiquidityFlag::Taker,
                });
                *client_event_counter += 1;

                if maker.qty == 0 {
                    event_sink.push_client_event(ClientEvent {
                        client_id: maker.client_id,
                        id: *client_event_counter,
                        timestamp: time,
                        order_id: maker_id,
                        order_side: maker.side,
                        kind: ClientEventType::Filled,
                        liquidity_flag: LiquidityFlag::Maker,
                    });
                    *client_event_counter += 1;
                    level.pop_front();
                } else {
                    event_sink.push_client_event(ClientEvent {
                        client_id: maker.client_id,
                        id: *client_event_counter,
                        timestamp: time,
                        order_id: maker_id,
                        order_side: maker.side,
                        kind: ClientEventType::PartiallyFilled(maker.qty),
                        liquidity_flag: LiquidityFlag::Maker,
                    });
                    *client_event_counter += 1;
                    break;
                }
            }
        }
    }

    /// Gets the total quantity at a given price level
    fn get_qty(&self, price: Price, side: OrderSide) -> u64 {
        match side {
            OrderSide::Ask => self
                .ask_orders
                .get(&price)
                .map(|level| level.total_qty)
                .unwrap_or(0),
            OrderSide::Bid => self
                .bid_orders
                .get(&price)
                .map(|level| level.total_qty)
                .unwrap_or(0),
        }
    }

    /// Executes a market order, emitting trade events for all trades made
    fn execute_market_order_and_emit_events(
        &mut self,
        order: Order,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        let mut market_order = LimitOrder::new(order);
        if market_order.qty == 0 {
            self.reject_order(order, time);
            return None;
        }
        self.match_order(&mut market_order, time);
        Some(market_order)
    }

    /// Checks the current state of the lob and updates cached value for best_ask and best_bid
    fn update_aggregates(&mut self) {
        self.best_ask = self.best_ask().unwrap_or(0);
        self.best_bid = self.best_bid().unwrap_or(0);
    }

    /// Emits a client event rejecting an order
    fn reject_order(&mut self, order: Order, time: Timestamp) {
        self.event_sink.push_client_event(ClientEvent {
            client_id: order.client_id,
            id: self.client_event_counter,
            timestamp: time,
            order_id: order.order_id,
            order_side: order.side,
            kind: ClientEventType::Rejected,
            liquidity_flag: LiquidityFlag::Invalid,
        });
        self.client_event_counter += 1;
    }

    /// Emits a client event accepting an order
    fn accept_order(&mut self, order: Order, time: Timestamp, qty: u32) {
        self.event_sink.push_client_event(ClientEvent {
            client_id: order.client_id,
            id: self.client_event_counter,
            timestamp: time,
            order_id: order.order_id,
            order_side: order.side,
            kind: ClientEventType::Accepted(qty),
            liquidity_flag: LiquidityFlag::Invalid,
        });
        self.client_event_counter += 1;
    }
}

/* UNIT TESTS */
#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::lob_core::market_events::{
        L3Event, NullFeeds, SeparateEventFeeds, SingleEventFeed,
    };
    use ringbuf::{HeapCons, HeapRb, traits::*};

    fn create_event_feeds(
        queue_size: usize,
    ) -> (
        SeparateEventFeeds,
        (
            HeapCons<L3Event>,
            HeapCons<TradeEvent>,
            HeapCons<ClientEvent>,
        ),
    ) {
        let (l3_prod, l3_cons) = HeapRb::<L3Event>::new(queue_size).split();
        let (t_prod, t_cons) = HeapRb::<TradeEvent>::new(queue_size).split();
        let (c_prod, c_cons) = HeapRb::<ClientEvent>::new(queue_size).split();
        (
            SeparateEventFeeds::new(l3_prod, t_prod, c_prod),
            (l3_cons, t_cons, c_cons),
        )
    }

    fn cancel_event<T: EventSink>(
        book: &mut OrderBook<T>,
        old_order_id: OrderId,
        side: OrderSide,
        timestamp: Timestamp,
    ) -> Option<LimitOrder> {
        book.process_order(Order::new(
            0,
            0,
            side,
            timestamp,
            OrderType::Cancel {
                old_id: old_order_id,
            },
        ))
    }

    #[test]
    fn empty_book_has_no_best_prices() {
        let mut book = OrderBook::new(NullFeeds {});
        assert!(book.best_bid().is_none());
        assert!(book.best_ask().is_none());
    }

    #[test]
    fn add_bid_without_crossing() {
        let mut book = OrderBook::new(NullFeeds {});
        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            1,
            OrderType::Limit { qty: 1, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 1, price: 200 },
        ));
        assert_eq!(book.best_bid(), Some(100));
        assert_eq!(book.best_ask(), Some(200));
    }

    #[test]
    fn cancel_removes_order() {
        let mut book = OrderBook::new(NullFeeds {});

        // TODO: Update depending on which id should be used to cancel orders
        book.process_order(Order::new(
            5,
            0,
            OrderSide::Bid,
            1,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        assert!(
            book.process_order(Order::new(
                0,
                0,
                OrderSide::Bid,
                1,
                OrderType::Cancel { old_id: 0 }
            ))
            .is_some()
        );
        assert!(book.best_bid().is_none());
    }

    #[test]
    fn pruning_multiple_price_levels() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..=2 {
            book.process_order(Order::new(
                0,
                i,
                OrderSide::Bid,
                i,
                OrderType::Limit {
                    qty: 5,
                    price: 100 + 5 * i as Price,
                },
            ));
        }
        assert_eq!(book.best_bid(), Some(110));
        for i in 3..=4 {
            assert!(cancel_event(&mut book, 5 - i, OrderSide::Bid, i).is_some());
        }
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn cancel_nonexistent_returns_none() {
        let mut book = OrderBook::new(NullFeeds {});
        assert!(cancel_event(&mut book, 0, OrderSide::Bid, 0).is_none());
    }

    #[test]
    fn update_order_updates_order() {
        let mut book = OrderBook::new(NullFeeds {});
        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            1,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        assert_eq!(book.best_bid(), Some(100));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Bid,
            1,
            OrderType::Update {
                old_id: 0,
                qty: 5,
                price: 500,
            },
        ));
        assert_eq!(book.best_bid(), Some(500));
    }

    #[test]
    fn update_nonexistent_order_has_no_effect() {
        let mut book = OrderBook::new(NullFeeds {});
        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            1,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        assert_eq!(book.best_bid(), Some(100));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Bid,
            1,
            OrderType::Update {
                old_id: 1,
                qty: 5,
                price: 500,
            },
        ));
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn best_bid_is_highest_price() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..=2 {
            book.process_order(Order::new(
                0,
                i,
                OrderSide::Bid,
                i,
                OrderType::Limit {
                    qty: 5,
                    price: 100 + 5 * i as Price,
                },
            ));
        }
        assert_eq!(book.best_bid(), Some(110));
    }

    #[test]
    fn many_orders_do_not_panic() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..1_000_000 {
            book.process_order(Order::new(
                0,
                i,
                OrderSide::Bid,
                i,
                OrderType::Limit {
                    qty: 10,
                    price: 100 + (i % 10) as Price,
                },
            ));
        }

        assert!(book.best_bid().is_some());
    }

    #[test]
    fn fifo_within_price_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, mut client_feed) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        for i in 0..2 {
            book.process_order(Order::new(
                0,
                i,
                OrderSide::Ask,
                i,
                OrderType::Limit { qty: 5, price: 100 },
            ));
        }

        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Limit { qty: 6, price: 100 },
        ));

        let trade_0 = client_feed.try_pop().unwrap();
        let trade_1 = client_feed.try_pop().unwrap();
        assert_eq!(trade_0.order_id, 0);
        assert_eq!(trade_1.order_id, 1);
    }

    #[test]
    fn simple_full_match() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (_, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 100 },
        ));

        let event_0 = trade_events.try_pop().unwrap();
        assert!(trade_events.try_pop().is_none());
        assert_eq!(event_0.quantity, 5);
        assert_eq!(event_0.price, 100);
        assert_eq!(event_0.aggressor_side, OrderSide::Ask);
        assert!(book.best_bid().is_none());
        assert!(book.best_ask().is_none());
    }

    #[test]
    fn partial_match_leaves_resting_qty() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (_, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 3, price: 100 },
        ));
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 3);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Ask);

        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn multi_level_sweep() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 105 },
        ));
        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Limit { qty: 6, price: 105 },
        ));
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 1);
        assert_eq!(trade_1.price, 105);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        assert_eq!(book.best_ask(), Some(105));
    }

    #[test]
    fn market_order_single_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Bid,
            1,
            OrderType::Market { qty: 5 },
        ));
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn market_order_multi_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 150 },
        ));
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Market { qty: 9 },
        ));
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 4);
        assert_eq!(trade_1.price, 150);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        assert_eq!(book.best_ask().unwrap(), 150);
    }

    #[test]
    fn market_order_partial_fill() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 150 },
        ));
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Market { qty: 15 },
        ));

        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 5);
        assert_eq!(trade_1.price, 150);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn market_order_no_fill() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (_, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Bid,
            0,
            OrderType::Market { qty: 15 },
        ));

        assert!(trade_events.try_pop().is_none());

        assert!(client_events.try_pop().is_none());

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn no_zero_trade_events() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 150 },
        ));
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Market { qty: 1 },
        ));

        while let Some(trade) = trade_events.try_pop() {
            assert!(trade.quantity != 0);
        }
    }

    #[test]
    fn limit_order_correct_qty_after_trades() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (mut l3_events, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(Order::new(
            0,
            0,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        book.process_order(Order::new(
            0,
            1,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 150 },
        ));
        book.process_order(Order::new(
            0,
            2,
            OrderSide::Bid,
            2,
            OrderType::Limit {
                qty: 15,
                price: 150,
            },
        ));

        let trade_event_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_event_0.maker_id, 0);
        assert_eq!(trade_event_0.price, 100);
        assert_eq!(trade_event_0.quantity, 5);
        assert_eq!(trade_event_0.aggressor_side, OrderSide::Bid);

        let trade_event_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_event_1.maker_id, 1);
        assert_eq!(trade_event_1.price, 150);
        assert_eq!(trade_event_1.quantity, 5);
        assert_eq!(trade_event_1.aggressor_side, OrderSide::Bid);

        l3_events.try_pop(); // discard L3 event for order 0 being accepted
        l3_events.try_pop(); // discard L3 event for order 1 being accepted
        let l3_event_0 = l3_events.try_pop().unwrap();
        assert_eq!(l3_event_0.order_id, 2);
        assert_eq!(l3_event_0.side, OrderSide::Bid);
        assert_eq!(l3_event_0.timestamp, 2);
        assert_eq!(l3_event_0.kind, OrderType::Limit { qty: 5, price: 150 });
    }

    #[test]
    fn test_execution_reports() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        let trade_0_clid = 5;
        let trade_0_id = 0;
        book.process_order(Order::new(
            // Add order
            trade_0_clid,
            trade_0_id,
            OrderSide::Ask,
            0,
            OrderType::Limit { qty: 5, price: 100 },
        ));
        let update_id = 1;
        book.process_order(Order::new(
            // Add order
            6,
            update_id,
            OrderSide::Ask,
            1,
            OrderType::Limit { qty: 5, price: 150 },
        ));
        let cancel_id = 2;
        book.process_order(Order::new(
            // Add order
            7,
            cancel_id,
            OrderSide::Ask,
            2,
            OrderType::Limit { qty: 5, price: 10 },
        ));
        book.process_order(Order::new(
            // Cancel order
            8,
            3,
            OrderSide::Ask,
            3,
            OrderType::Cancel { old_id: cancel_id },
        ));
        let trade_1_clid = 9;
        let trade_1_id = 4;
        book.process_order(Order::new(
            // Update order
            trade_1_clid,
            trade_1_id,
            OrderSide::Ask,
            4,
            OrderType::Update {
                old_id: update_id,
                qty: 10,
                price: 150,
            },
        ));
        let trade_maker_clid = 10;
        let trade_maker_id = 5;
        book.process_order(Order::new(
            // Make trades
            trade_maker_clid,
            trade_maker_id,
            OrderSide::Bid,
            5,
            OrderType::Limit {
                qty: 15,
                price: 150,
            },
        ));

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.kind, ClientEventType::Accepted(5));
        assert_eq!(event.timestamp, 0);
        let event = client_events.try_pop().unwrap();
        assert_eq!(event.kind, ClientEventType::Accepted(5));
        assert_eq!(event.timestamp, 1);
        let event = client_events.try_pop().unwrap();
        assert_eq!(event.kind, ClientEventType::Accepted(5));
        assert_eq!(event.timestamp, 2);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 3);
        assert_eq!(event.client_id, 8);
        assert_eq!(event.order_id, 3);
        assert_eq!(event.kind, ClientEventType::Canceled);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 4);
        assert_eq!(event.client_id, 9);
        assert_eq!(event.order_id, 4);
        assert_eq!(event.kind, ClientEventType::Updated);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.kind, ClientEventType::Accepted(15));
        assert_eq!(event.timestamp, 5);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 5);
        assert_eq!(event.client_id, trade_maker_clid);
        assert_eq!(event.order_id, trade_maker_id);
        assert_eq!(event.order_side, OrderSide::Bid);
        assert_eq!(event.kind, ClientEventType::PartiallyFilled(10));
        assert_eq!(event.liquidity_flag, LiquidityFlag::Taker);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 5);
        assert_eq!(event.client_id, trade_0_clid);
        assert_eq!(event.order_id, trade_0_id);
        assert_eq!(event.order_side, OrderSide::Ask);
        assert_eq!(event.kind, ClientEventType::Filled);
        assert_eq!(event.liquidity_flag, LiquidityFlag::Maker);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 5);
        assert_eq!(event.client_id, trade_maker_clid);
        assert_eq!(event.order_id, trade_maker_id);
        assert_eq!(event.order_side, OrderSide::Bid);
        assert_eq!(event.kind, ClientEventType::Filled);
        assert_eq!(event.liquidity_flag, LiquidityFlag::Taker);

        let event = client_events.try_pop().unwrap();
        assert_eq!(event.timestamp, 5);
        assert_eq!(event.client_id, trade_1_clid);
        assert_eq!(event.order_id, trade_1_id);
        assert_eq!(event.order_side, OrderSide::Ask);
        assert_eq!(event.kind, ClientEventType::Filled);
        assert_eq!(event.liquidity_flag, LiquidityFlag::Maker);

        assert!(client_events.try_pop().is_none());
    }
}
