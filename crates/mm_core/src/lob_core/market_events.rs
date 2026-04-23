use crate::lob_core::{
    ClientId, OrderId, OrderQty, Price, Timestamp,
    market_orders::{LimitOrder, Order, OrderSide, OrderType},
};
use ringbuf::{HeapProd, traits::Producer};

/// Event type representing a single L3 data point, ie an individual order
/// Emitted on every order received by the limit order book
// pub type L3Event = Order;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct L3Event {
    pub order_id: OrderId,
    pub side: OrderSide,
    pub timestamp: Timestamp,
    pub kind: OrderType,
    pub extra: L3EventExtra,
}
impl L3Event {
    pub fn new(order: Order, extra: L3EventExtra) -> Self {
        Self {
            order_id: order.order_id,
            side: order.side,
            timestamp: order.timestamp,
            kind: order.kind,
            extra,
        }
    }
    /// Constructors for different order types for ease of use
    pub fn new_limit(order: LimitOrder, timestamp: Timestamp) -> Self {
        let order = Order::new(
            0, // NOTE: These don't get sent by moldudp64, so use a junk value that gets discarded
            order.order_id,
            order.side,
            timestamp,
            OrderType::Limit {
                qty: order.qty,
                price: order.price,
            },
        );
        Self::new(order, L3EventExtra::None)
    }
    pub fn new_update(order: Order) -> Self {
        Self::new(order, L3EventExtra::None)
    }
    pub fn new_cancel(order: Order, old_qty: OrderQty) -> Self {
        Self::new(order, L3EventExtra::Cancel(old_qty))
    }
}

/// Stores quantity for canceled events
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum L3EventExtra {
    Cancel(OrderQty),
    None,
}

/// Event type representing an executed trade
/// Emitted every time a trade is executed
#[derive(Copy, Clone, Debug)]
pub struct TradeEvent {
    pub price: Price,
    pub quantity: OrderQty,
    pub aggressor_side: OrderSide,
    pub maker_id: OrderId,
}

/// Event type for sending specific information regarding market events to clients involved
/// For example, after a trade, these will be sent to the two clients that executed the trade
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientEventType {
    Accepted(OrderQty),
    Rejected,
    Updated,
    Canceled,
    // Contains the unfilled quantity
    PartiallyFilled(OrderQty),
    Filled,
}

/// Flag denoting if an event corresponds to a maker or taker
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LiquidityFlag {
    Maker,
    Taker,
    Invalid,
}

/// Special event that is only sent to client involved in a particular event
/// These events are sent via FIX, so they are not included in the MarketEvent struct and instead have their own struct
/// For example, if a trade is made then the owner of the filled order will be sent a ClientEvent
#[derive(Copy, Clone, Debug)]
pub struct ClientEvent {
    pub id: u64,
    pub timestamp: Timestamp,
    pub client_id: ClientId,
    pub order_id: OrderId,
    pub order_side: OrderSide,
    pub kind: ClientEventType,
    pub liquidity_flag: LiquidityFlag,
}

/// Generic market event struct, encompasses all types of market events
#[derive(Copy, Clone, Debug)]
pub struct MarketEvent {
    pub id: u16,
    pub timestamp: Timestamp,
    pub kind: MarketEventType,
}
impl MarketEvent {
    pub fn new(id: u16, timestamp: Timestamp, kind: MarketEventType) -> Self {
        Self {
            id,
            timestamp,
            kind,
        }
    }
    pub fn new_limit(id: u16, timestamp: Timestamp, order: LimitOrder) -> Self {
        Self {
            id,
            timestamp,
            kind: MarketEventType::L3(L3Event::new_limit(order, timestamp)),
        }
    }
    pub fn new_update(id: u16, timestamp: Timestamp, order: Order) -> Self {
        Self {
            id,
            timestamp,
            kind: MarketEventType::L3(L3Event::new_update(order)),
        }
    }
    pub fn new_cancel(id: u16, timestamp: Timestamp, order: Order, old_qty: OrderQty) -> Self {
        Self {
            id,
            timestamp,
            kind: MarketEventType::L3(L3Event::new_cancel(order, old_qty)),
        }
    }
}

/// Enum containing type specific information about a MarketEvent
#[derive(Copy, Clone, Debug)]
pub enum MarketEventType {
    L3(L3Event),
    Trade(TradeEvent),
}

/// Impl for structs defining a way for a OrderBook to emit events
pub trait EventSink {
    fn push_event(&mut self, event: MarketEvent);
    fn push_client_event(&mut self, event: ClientEvent);
}

/// Event Feed struct containing separate queues for each type of market event
/// push() routes the event into the specific feed
pub struct SeparateEventFeeds {
    l3_events: HeapProd<L3Event>,
    trade_events: HeapProd<TradeEvent>,
    client_events: HeapProd<ClientEvent>,
}
impl SeparateEventFeeds {
    pub fn new(
        l3_events: HeapProd<L3Event>,
        trade_events: HeapProd<TradeEvent>,
        client_events: HeapProd<ClientEvent>,
    ) -> Self {
        Self {
            l3_events,
            trade_events,
            client_events,
        }
    }
}
impl EventSink for SeparateEventFeeds {
    /// Matches a market event to the correct feed based on type and pushes
    /// Blocks until able to be pushed into feed
    fn push_event(&mut self, event: MarketEvent) {
        match event.kind {
            MarketEventType::L3(event) => while self.l3_events.try_push(event).is_err() {},
            MarketEventType::Trade(event) => while self.trade_events.try_push(event).is_err() {},
        }
    }
    fn push_client_event(&mut self, event: ClientEvent) {
        while self.client_events.try_push(event).is_err() {}
    }
}

/// Event feed struct that drops all events pushed into it
pub struct NullFeeds {}
impl EventSink for NullFeeds {
    fn push_event(&mut self, _: MarketEvent) {}
    fn push_client_event(&mut self, _: ClientEvent) {}
}

/// Event feed struct with a single unified queue for pushing events into
/// Specific event types should be parsed out later in the pipeline
pub struct SingleEventFeed {
    events: HeapProd<MarketEvent>,
    client_events: HeapProd<ClientEvent>,
}
impl SingleEventFeed {
    pub fn new(events: HeapProd<MarketEvent>, client_events: HeapProd<ClientEvent>) -> Self {
        Self {
            events,
            client_events,
        }
    }
}
impl EventSink for SingleEventFeed {
    fn push_event(&mut self, event: MarketEvent) {
        while self.events.try_push(event).is_err() {}
    }
    fn push_client_event(&mut self, event: ClientEvent) {
        while self.client_events.try_push(event).is_err() {}
    }
}
