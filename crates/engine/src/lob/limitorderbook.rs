use crate::lob::market_events::{
    ClientEvent, ClientEventType, EventSink, L1Event, L2Event, LiquidityFlag, MarketEvent,
    MarketEventType, TradeEvent,
};
use crate::lob::order::{self, LimitOrder, Order, OrderSide, OrderStatus, OrderType};
use crate::lob::types::{OrderId, Price, Timestamp};
use std::collections::{BTreeMap, HashMap, VecDeque};

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
                    self.total_qty -= order.qty;
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
        self.total_qty += order.qty;
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
        }
    }
    /// Accepts an Order and handles it according to its OrderType
    ///
    /// LimitOrders are matched and added into LOB if not completely matched
    /// MarketOrders attempt to make qty trades starting from best price and partially fill
    ///   if there is not enough liquidity
    /// CancelOrders attempt to cancel an order
    /// UpdateOrders cancel the previously existing order and resubmit a new order
    pub fn process_order(&mut self, order: Order, time: Timestamp) -> Option<LimitOrder> {
        // TODO: Update return type to be more informative
        self.event_sink.push(MarketEvent {
            timestamp: time,
            kind: MarketEventType::L3(order),
        });
        let order = match order.kind {
            OrderType::Limit { qty, price } => {
                let order = self.add_order_and_emit_events(LimitOrder::new(order), time);
                Some(order)
            }
            OrderType::Market { qty } => {
                let mut order = LimitOrder::new(order);
                self.match_order(&mut order, time);
                Some(order)
            }
            OrderType::Cancel => self.cancel_order_and_emit_events(order.order_id, time),
            OrderType::Update { old_id, qty, price } => {
                self.update_order(LimitOrder::new(order), old_id, time)
            }
        };
        self.generate_l1_events(time);
        order
    }

    /// Prunes lazily removed bid orders and returns the current best bid
    /// Does not update the cached value of best bid
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
        original_order: LimitOrder,
        time: Timestamp,
    ) -> LimitOrder {
        let mut order = original_order;
        self.match_order(&mut order, time);
        if order.qty == 0 {
            return order;
        }
        let level = match order.side {
            OrderSide::Bid => {
                self.total_bids += order.qty;
                self.bid_orders.entry(order.price).or_default()
            }
            OrderSide::Ask => {
                self.total_asks += order.qty;
                self.ask_orders.entry(order.price).or_default()
            }
        };
        level.push(&order);
        self.orders.insert(order.order_id, order);

        self.event_sink.push(MarketEvent::new(
            time,
            MarketEventType::L2(L2Event {
                price: (order.price),
                side: order.side,
                level_size: level.total_qty,
                total_size: match order.side {
                    OrderSide::Ask => self.total_asks,
                    OrderSide::Bid => self.total_bids,
                },
            }),
        ));
        self.event_sink.push(MarketEvent::new(
            time,
            MarketEventType::Client(ClientEvent {
                order_id: order.order_id,
                kind: ClientEventType::Accepted,
                liquidity_flag: LiquidityFlag::Invalid,
            }),
        ));
        order
    }

    // /// Executes a trade if a valid match can be made, see match_order() for details about matching.
    // /// Adds an order to the side of the book specified in the order if any of the order's quantity is unmatched.
    // /// Does not emit any events
    // fn add_order(&mut self, original_order: LimitOrder) -> LimitOrder {
    //     let mut order = original_order;
    //     self.match_order(&mut order);
    //     if order.qty == 0 {
    //         return order;
    //     }
    //     let level = match order.side {
    //         OrderSide::Bid => {
    //             self.total_bids += order.qty;
    //             self.bid_orders.entry(order.price).or_default()
    //         }
    //         OrderSide::Ask => {
    //             self.total_asks += order.qty;
    //             self.ask_orders.entry(order.price).or_default()
    //         }
    //     };
    //     level.push(&order);
    //     self.orders.insert(order.order_id, order);
    //     order
    // }

    /// Updates an existing order by cancelling it and replacing it with a new order. Executes
    /// a trade if a valid match can be made
    fn update_order(
        &mut self,
        order: LimitOrder,
        old_order_id: OrderId,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        let x = self
            .cancel_order(old_order_id)
            .map(|_| self.add_order_and_emit_events(order, time));
        if let Some(_) = x {
            self.event_sink.push(MarketEvent::new(
                time,
                MarketEventType::Client(ClientEvent {
                    order_id: old_order_id,
                    kind: ClientEventType::Updated,
                    liquidity_flag: LiquidityFlag::Invalid,
                }),
            ));
        } else {
            self.event_sink.push(MarketEvent::new(
                time,
                MarketEventType::Client(ClientEvent {
                    order_id: old_order_id,
                    kind: ClientEventType::Rejected,
                    liquidity_flag: LiquidityFlag::Invalid,
                }),
            ));
        }
        x
    }

    /// Lazily cancels an order by marking it as canceled. Lazily canceled orders are pruned
    /// by `best_bid()`, `best_ask()`, or `match_order()`
    /// Emits a cancellation event if order_id points to a valid order
    fn cancel_order_and_emit_events(
        &mut self,
        order_id: OrderId,
        time: Timestamp,
    ) -> Option<LimitOrder> {
        match self.orders.get_mut(&order_id) {
            Some(order) => {
                let (level, total_qty) = match order.side {
                    OrderSide::Ask => {
                        self.total_asks -= order.qty;
                        (
                            self.ask_orders.get_mut(&order.price).unwrap(), // If order is Some() then the price level its supposed to exist in should always exist
                            self.total_asks,
                        )
                    }
                    OrderSide::Bid => {
                        self.total_bids -= order.qty;
                        (
                            self.bid_orders.get_mut(&order.price).unwrap(), // If order is Some() then the price level its supposed to exist in should always exist
                            self.total_bids,
                        )
                    }
                };
                level.total_qty -= order.qty;
                order.qty = 0;
                order.status = OrderStatus::Canceled;
                self.event_sink.push(MarketEvent::new(
                    time,
                    MarketEventType::L2(L2Event {
                        price: order.price,
                        side: order.side,
                        level_size: level.total_qty,
                        total_size: total_qty,
                    }),
                ));
                self.event_sink.push(MarketEvent::new(
                    time,
                    MarketEventType::Client(ClientEvent {
                        order_id,
                        kind: ClientEventType::Canceled,
                        liquidity_flag: LiquidityFlag::Invalid,
                    }),
                ));
                Some(*order)
            }
            None => {
                self.event_sink.push(MarketEvent::new(
                    time,
                    MarketEventType::Client(ClientEvent {
                        order_id,
                        kind: ClientEventType::Rejected,
                        liquidity_flag: LiquidityFlag::Invalid,
                    }),
                ));
                None
            }
        }
    }

    /// Lazily cancels an order by marking it as canceled. Lazily canceled orders are pruned
    /// by `best_bid()`, `best_ask()`, or `match_order()`
    /// Emits no events, even if order_id points to a valid order
    fn cancel_order(&mut self, order_id: OrderId) -> Option<LimitOrder> {
        if let Some(order) = self.orders.get_mut(&order_id) {
            if order.status == OrderStatus::Canceled || order.qty == 0 {
                return None;
            }
            let level = match order.side {
                OrderSide::Ask => {
                    self.total_asks -= order.qty;
                    // self.ask_orders.get_mut(&order.price).unwrap() // If order is Some() then the price level its supposed to exist in should always exist
                    let test = self.ask_orders.get_mut(&order.price);
                    if test.is_none() {
                        println!("Oh great.");
                        println!("{:?}", order);
                        println!("{:?}", test);
                    }
                    test.unwrap()
                }
                OrderSide::Bid => {
                    self.total_bids -= order.qty;
                    // self.bid_orders.get_mut(&order.price).unwrap() // If order is Some() then the price level its supposed to exist in should always exist
                    let test = self.bid_orders.get_mut(&order.price);
                    if test.is_none() {
                        println!("Oh great.");
                        println!("{:?}", order);
                        println!("{:?}", test);
                    }
                    test.unwrap()
                }
            };
            level.total_qty -= order.qty;
            order.qty = 0;
            order.status = OrderStatus::Canceled;
            Some(*order)
        } else {
            None
        }
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
            ),
            OrderSide::Ask => Self::make_trades(
                self.bid_orders.iter_mut().rev(),
                &mut self.orders,
                &mut self.event_sink,
                order,
                time,
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
    ) {
        let initial_qty = taker.qty;
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
                let trade_volume = std::cmp::min(maker.qty, taker.qty);
                maker.qty -= trade_volume;
                taker.qty -= trade_volume;

                event_sink.push(MarketEvent::new(
                    time,
                    MarketEventType::Trade(TradeEvent {
                        price: *price,
                        quantity: trade_volume,
                        aggressor_side: taker.side,
                    }),
                ));

                if maker.qty == 0 {
                    event_sink.push(MarketEvent::new(
                        time,
                        MarketEventType::Client(ClientEvent {
                            order_id: maker_id,
                            kind: ClientEventType::Filled,
                            liquidity_flag: LiquidityFlag::Maker,
                        }),
                    ));
                    level.pop_front();
                } else {
                    event_sink.push(MarketEvent::new(
                        time,
                        MarketEventType::Client(ClientEvent {
                            order_id: maker_id,
                            kind: ClientEventType::PartiallyFilled(maker.qty),
                            liquidity_flag: LiquidityFlag::Maker,
                        }),
                    ));
                    break;
                }
            }
        }
        if taker.qty == initial_qty {
            return;
        }
        event_sink.push(MarketEvent::new(
            time,
            MarketEventType::Client(ClientEvent {
                order_id: taker.order_id,
                kind: match taker.qty == 0 {
                    true => ClientEventType::Filled,
                    false => ClientEventType::PartiallyFilled(taker.qty),
                },
                liquidity_flag: LiquidityFlag::Taker,
            }),
        ));
    }

    /// Gets the total quantity at a given price level
    fn get_qty(&self, price: u64, side: OrderSide) -> u64 {
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

    /// Checks the current state of the lob and generates L1/L2 events if applicable
    /// Updates cached value for best_ask and best_bid
    fn generate_l1_events(&mut self, time: Timestamp) {
        let new_best_ask = self.best_ask().unwrap_or(0);
        let new_best_bid = self.best_bid().unwrap_or(0);
        if new_best_ask != self.best_ask {
            self.best_ask = new_best_ask;
            self.event_sink.push(MarketEvent::new(
                time,
                MarketEventType::L1(L1Event {
                    price: self.best_ask,
                    side: OrderSide::Ask,
                    size: self.get_qty(self.best_ask, OrderSide::Ask),
                }),
            ));
        }
        if new_best_bid != self.best_bid {
            self.best_bid = new_best_bid;
            self.event_sink.push(MarketEvent::new(
                time,
                MarketEventType::L1(L1Event {
                    price: self.best_bid,
                    side: OrderSide::Bid,
                    size: self.get_qty(self.best_bid, OrderSide::Bid),
                }),
            ));
        }
    }
}

/* UNIT TESTS */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lob::market_events::{ClientEvent, L3Event, NullFeeds, SeparateEventFeeds};
    use ringbuf::{HeapCons, HeapRb, traits::*};

    fn create_event_feeds(
        queue_size: usize,
    ) -> (
        SeparateEventFeeds,
        (
            HeapCons<L1Event>,
            HeapCons<L2Event>,
            HeapCons<L3Event>,
            HeapCons<TradeEvent>,
            HeapCons<ClientEvent>,
        ),
    ) {
        let (l1_prod, l1_cons) = HeapRb::<L1Event>::new(queue_size).split();
        let (l2_prod, l2_cons) = HeapRb::<L2Event>::new(queue_size).split();
        let (l3_prod, l3_cons) = HeapRb::<L3Event>::new(queue_size).split();
        let (t_prod, t_cons) = HeapRb::<TradeEvent>::new(queue_size).split();
        let (c_prod, c_cons) = HeapRb::<ClientEvent>::new(queue_size).split();
        (
            SeparateEventFeeds::new(l1_prod, l2_prod, l3_prod, t_prod, c_prod),
            (l1_cons, l2_cons, l3_cons, t_cons, c_cons),
        )
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
        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                1,
                OrderType::Limit { qty: 1, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 1, price: 200 },
            ),
            0,
        );
        assert_eq!(book.best_bid(), Some(100));
        assert_eq!(book.best_ask(), Some(200));
    }

    #[test]
    fn cancel_removes_order() {
        let mut book = OrderBook::new(NullFeeds {});

        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                1,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        assert!(
            book.process_order(Order::new(0, OrderSide::Bid, 1, OrderType::Cancel), 0)
                .is_some()
        );
        assert!(book.best_bid().is_none());
    }

    #[test]
    fn pruning_multiple_price_levels() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..=2 {
            book.process_order(
                Order::new(
                    i,
                    OrderSide::Bid,
                    i,
                    OrderType::Limit {
                        qty: 5,
                        price: 100 + 5 * i,
                    },
                ),
                0,
            );
        }
        assert_eq!(book.best_bid(), Some(110));
        assert!(book.cancel_order(1).is_some());
        assert!(book.cancel_order(2).is_some());
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn cancel_nonexistent_returns_none() {
        let mut book = OrderBook::new(NullFeeds {});
        assert!(
            book.process_order(Order::new(0, OrderSide::Bid, 1, OrderType::Cancel), 0)
                .is_none()
        );
    }

    #[test]
    fn update_order_updates_order() {
        let mut book = OrderBook::new(NullFeeds {});
        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                1,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        assert_eq!(book.best_bid(), Some(100));
        book.process_order(
            Order::new(
                1,
                OrderSide::Bid,
                1,
                OrderType::Update {
                    old_id: 0,
                    qty: 5,
                    price: 500,
                },
            ),
            0,
        );
        assert_eq!(book.best_bid(), Some(500));
    }

    #[test]
    fn update_nonexistent_order_has_no_effect() {
        let mut book = OrderBook::new(NullFeeds {});
        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                1,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        assert_eq!(book.best_bid(), Some(100));
        book.process_order(
            Order::new(
                1,
                OrderSide::Bid,
                1,
                OrderType::Update {
                    old_id: 1,
                    qty: 5,
                    price: 500,
                },
            ),
            0,
        );
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn best_bid_is_highest_price() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..=2 {
            book.process_order(
                Order::new(
                    i,
                    OrderSide::Bid,
                    i,
                    OrderType::Limit {
                        qty: 5,
                        price: 100 + 5 * i,
                    },
                ),
                0,
            );
        }
        assert_eq!(book.best_bid(), Some(110));
    }

    #[test]
    fn many_orders_do_not_panic() {
        let mut book = OrderBook::new(NullFeeds {});

        for i in 0..1_000_000 {
            book.process_order(
                Order::new(
                    i,
                    OrderSide::Bid,
                    i,
                    OrderType::Limit {
                        qty: 10,
                        price: 100 + (i % 10),
                    },
                ),
                0,
            );
        }

        assert!(book.best_bid().is_some());
    }

    #[test]
    fn fifo_within_price_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, _, mut client_feed) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        for i in 0..2 {
            book.process_order(
                Order::new(
                    i,
                    OrderSide::Ask,
                    i,
                    OrderType::Limit { qty: 5, price: 100 },
                ),
                0,
            );
        }

        book.process_order(
            Order::new(
                2,
                OrderSide::Bid,
                2,
                OrderType::Limit { qty: 6, price: 100 },
            ),
            0,
        );

        let trade_0 = client_feed.try_pop().unwrap();
        let trade_1 = client_feed.try_pop().unwrap();
        assert_eq!(trade_0.order_id, 0);
        assert_eq!(trade_1.order_id, 1);
    }

    #[test]
    fn simple_full_match() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (_, _, _, mut trade_events, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );

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
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Bid,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 3, price: 100 },
            ),
            0,
        );
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 3);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Ask);

        client_events.try_pop(); // Discard Accepted event
        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::PartiallyFilled(2));

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 1);
        assert_eq!(client_event_1.kind, ClientEventType::Filled);

        assert!(client_events.try_pop().is_none());

        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn multi_level_sweep() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 5, price: 105 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                2,
                OrderSide::Bid,
                2,
                OrderType::Limit { qty: 6, price: 105 },
            ),
            0,
        );
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 1);
        assert_eq!(trade_1.price, 105);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        client_events.try_pop(); // discard accepted events
        client_events.try_pop();

        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::Filled);
        assert_eq!(client_event_0.liquidity_flag, LiquidityFlag::Maker);

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 1);
        assert_eq!(client_event_1.kind, ClientEventType::PartiallyFilled(4));
        assert_eq!(client_event_1.liquidity_flag, LiquidityFlag::Maker);

        let client_event_2 = client_events.try_pop().unwrap();
        assert_eq!(client_event_2.order_id, 2);
        assert_eq!(client_event_2.kind, ClientEventType::Filled);
        assert_eq!(client_event_2.liquidity_flag, LiquidityFlag::Taker);

        assert!(client_events.try_pop().is_none());

        assert_eq!(book.best_ask(), Some(105));
    }

    #[test]
    fn market_order_single_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(1, OrderSide::Bid, 1, OrderType::Market { qty: 5 }),
            0,
        );
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        client_events.try_pop(); // discard accepted events
        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::Filled);

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 1);
        assert_eq!(client_event_1.kind, ClientEventType::Filled);

        assert!(client_events.try_pop().is_none());

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn market_order_multi_level() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 5, price: 150 },
            ),
            0,
        );
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(
            Order::new(2, OrderSide::Bid, 2, OrderType::Market { qty: 9 }),
            0,
        );
        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 4);
        assert_eq!(trade_1.price, 150);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        client_events.try_pop(); // discard accepted events
        client_events.try_pop();

        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::Filled);
        assert_eq!(client_event_0.liquidity_flag, LiquidityFlag::Maker);

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 1);
        assert_eq!(client_event_1.kind, ClientEventType::PartiallyFilled(1));
        assert_eq!(client_event_1.liquidity_flag, LiquidityFlag::Maker);

        let client_event_2 = client_events.try_pop().unwrap();
        assert_eq!(client_event_2.order_id, 2);
        assert_eq!(client_event_2.kind, ClientEventType::Filled);
        assert_eq!(client_event_2.liquidity_flag, LiquidityFlag::Taker);

        assert!(client_events.try_pop().is_none());

        assert_eq!(book.best_ask().unwrap(), 150);
    }

    #[test]
    fn market_order_partial_fill() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 5, price: 150 },
            ),
            0,
        );
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(
            Order::new(2, OrderSide::Bid, 2, OrderType::Market { qty: 15 }),
            0,
        );

        let trade_0 = trade_events.try_pop().unwrap();
        assert_eq!(trade_0.quantity, 5);
        assert_eq!(trade_0.price, 100);
        assert_eq!(trade_0.aggressor_side, OrderSide::Bid);

        let trade_1 = trade_events.try_pop().unwrap();
        assert_eq!(trade_1.quantity, 5);
        assert_eq!(trade_1.price, 150);
        assert_eq!(trade_1.aggressor_side, OrderSide::Bid);

        client_events.try_pop(); // drop accepted events
        client_events.try_pop();

        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::Filled);
        assert_eq!(client_event_0.liquidity_flag, LiquidityFlag::Maker);

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 1);
        assert_eq!(client_event_1.kind, ClientEventType::Filled);
        assert_eq!(client_event_1.liquidity_flag, LiquidityFlag::Maker);

        let client_event_2 = client_events.try_pop().unwrap();
        assert_eq!(client_event_2.order_id, 2);
        assert_eq!(client_event_2.kind, ClientEventType::PartiallyFilled(5));
        assert_eq!(client_event_2.liquidity_flag, LiquidityFlag::Taker);

        assert!(client_events.try_pop().is_none());

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn market_order_no_fill() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(0, OrderSide::Bid, 0, OrderType::Market { qty: 15 }),
            0,
        );

        assert!(trade_events.try_pop().is_none());

        assert!(client_events.try_pop().is_none());

        assert!(book.best_ask().is_none());
    }

    #[test]
    fn best_price_change_emits_l1_event() {
        let (event_feeds, consumer_feeds) = create_event_feeds(4);
        let (mut l1_events, _, _, _, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(1, OrderSide::Ask, 1, OrderType::Limit { qty: 3, price: 90 }),
            0,
        );
        book.process_order(
            Order::new(2, OrderSide::Bid, 2, OrderType::Limit { qty: 6, price: 50 }),
            0,
        );
        book.process_order(
            Order::new(3, OrderSide::Bid, 3, OrderType::Limit { qty: 4, price: 75 }),
            0,
        );

        let event_0 = l1_events.try_pop().unwrap();
        assert_eq!(event_0.side, OrderSide::Ask);
        assert_eq!(event_0.price, 100);
        assert_eq!(event_0.size, 5);

        let event_1 = l1_events.try_pop().unwrap();
        assert_eq!(event_1.side, OrderSide::Ask);
        assert_eq!(event_1.price, 90);
        assert_eq!(event_1.size, 3);

        let event_2 = l1_events.try_pop().unwrap();
        assert_eq!(event_2.side, OrderSide::Bid);
        assert_eq!(event_2.price, 50);
        assert_eq!(event_2.size, 6);

        let event_3 = l1_events.try_pop().unwrap();
        assert_eq!(event_3.side, OrderSide::Bid);
        assert_eq!(event_3.price, 75);
        assert_eq!(event_3.size, 4);
    }

    #[test]
    fn quantity_and_price_changes_emit_l2_events() {
        let (event_feeds, consumer_feeds) = create_event_feeds(16);
        let (_, mut l2_events, _, _, _) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 3, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                2,
                OrderSide::Ask,
                2,
                OrderType::Limit { qty: 1, price: 110 },
            ),
            0,
        );
        book.process_order(
            Order::new(3, OrderSide::Bid, 3, OrderType::Limit { qty: 4, price: 75 }),
            0,
        );
        book.process_order(
            Order::new(4, OrderSide::Bid, 4, OrderType::Limit { qty: 2, price: 75 }),
            0,
        );
        book.process_order(
            Order::new(5, OrderSide::Bid, 5, OrderType::Limit { qty: 4, price: 85 }),
            0,
        );

        let event_0 = l2_events.try_pop().unwrap();
        assert_eq!(event_0.side, OrderSide::Ask);
        assert_eq!(event_0.price, 100);
        assert_eq!(event_0.level_size, 5);
        assert_eq!(event_0.total_size, 5);

        let event_1 = l2_events.try_pop().unwrap();
        assert_eq!(event_1.side, OrderSide::Ask);
        assert_eq!(event_1.price, 100);
        assert_eq!(event_1.level_size, 8);
        assert_eq!(event_1.total_size, 8);

        let event_2 = l2_events.try_pop().unwrap();
        assert_eq!(event_2.side, OrderSide::Ask);
        assert_eq!(event_2.price, 110);
        assert_eq!(event_2.level_size, 1);
        assert_eq!(event_2.total_size, 9);

        let event_3 = l2_events.try_pop().unwrap();
        assert_eq!(event_3.side, OrderSide::Bid);
        assert_eq!(event_3.price, 75);
        assert_eq!(event_3.level_size, 4);
        assert_eq!(event_3.total_size, 4);

        let event_4 = l2_events.try_pop().unwrap();
        assert_eq!(event_4.side, OrderSide::Bid);
        assert_eq!(event_4.price, 75);
        assert_eq!(event_4.level_size, 6);
        assert_eq!(event_4.total_size, 6);

        let event_5 = l2_events.try_pop().unwrap();
        assert_eq!(event_5.side, OrderSide::Bid);
        assert_eq!(event_5.price, 85);
        assert_eq!(event_5.level_size, 4);
        assert_eq!(event_5.total_size, 10);
    }

    #[test]
    fn no_zero_trade_events() {
        let (event_feeds, consumer_feeds) = create_event_feeds(32);
        let (_, _, _, mut trade_events, mut client_events) = consumer_feeds;
        let mut book = OrderBook::new(event_feeds);

        book.process_order(
            Order::new(
                0,
                OrderSide::Ask,
                0,
                OrderType::Limit { qty: 5, price: 100 },
            ),
            0,
        );
        book.process_order(
            Order::new(
                1,
                OrderSide::Ask,
                1,
                OrderType::Limit { qty: 5, price: 150 },
            ),
            0,
        );
        assert_eq!(book.best_ask(), Some(100));
        book.process_order(
            Order::new(2, OrderSide::Bid, 2, OrderType::Market { qty: 1 }),
            0,
        );

        while let Some(trade) = trade_events.try_pop() {
            assert!(trade.quantity != 0);
        }

        client_events.try_pop(); // drop accepted events
        client_events.try_pop();

        let client_event_0 = client_events.try_pop().unwrap();
        assert_eq!(client_event_0.order_id, 0);
        assert_eq!(client_event_0.kind, ClientEventType::PartiallyFilled(4));
        assert_eq!(client_event_0.liquidity_flag, LiquidityFlag::Maker);

        let client_event_1 = client_events.try_pop().unwrap();
        assert_eq!(client_event_1.order_id, 2);
        assert_eq!(client_event_1.kind, ClientEventType::Filled);
        assert_eq!(client_event_1.liquidity_flag, LiquidityFlag::Taker);

        assert!(client_events.try_pop().is_none());
    }
}
