use crate::lob::order::OrderSide;
use crate::lob::types::{OrderId, Price};
use crate::lob::{order::LimitOrder, types::Timestamp};
use ringbuf::{HeapProd, traits::*};

/// Event type representing a single L1 datapoint, ie information about the top of the book
/// Emitted when the best price or size of the best price level changes
#[derive(Copy, Clone, Debug)]
pub struct L1Event {
    pub price: Price,
    pub side: OrderSide,
    pub size: u64,
}

/// Event type representing a single L2 datapoint, ie information about a price level
/// Emitted when size or price level changes
#[derive(Copy, Clone, Debug)]
pub struct L2Event {
    pub price: Price,
    pub side: OrderSide,
    pub level_size: u64,
    pub total_size: u64,
}

/// Event type representing a single L3 data point, ie an individual order
/// Emitted on every order received by the limit order book
pub type L3Event = LimitOrder;

/// Event type representing an executed trade
/// Emitted every time a trade is executed
#[derive(Copy, Clone, Debug)]
pub struct TradeEvent {
    pub price: Price,
    pub quantity: u64,
    pub aggressor_side: OrderSide,
}

/// Event type for sending specific information regarding market events to clients involved
/// For example, after a trade, these will be sent to the two clients that executed the trade
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientEventType {
    Accepted,
    Rejected,
    Updated,
    Canceled,
    // Contains the unfilled quantity
    PartiallyFilled(u64),
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
/// For example, if a trade is made then the owner of the filled order will be sent a ClientEvent
#[derive(Copy, Clone, Debug)]
pub struct ClientEvent {
    pub order_id: OrderId,
    pub kind: ClientEventType,
    pub liquidity_flag: LiquidityFlag,
}

/// Generic market event struct, encompasses all types of market events
#[derive(Copy, Clone, Debug)]
pub struct MarketEvent {
    pub timestamp: Timestamp,
    pub kind: MarketEventType,
}
impl MarketEvent {
    pub fn new(kind: MarketEventType) -> Self {
        Self {
            timestamp: 0, // TODO: Implement proper time system!
            kind,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarketEventType {
    L1(L1Event),
    L2(L2Event),
    L3(L3Event),
    Trade(TradeEvent),
    Client(ClientEvent),
}

/// Impl for structs defining a way for a OrderBook to emit events
pub trait EventSink {
    fn push(&mut self, event: MarketEvent);
}

/// Event Feed struct containing separate queues for each type of market event
/// push() routes the event into the specific feed
pub struct SeparateEventFeeds {
    l1_events: HeapProd<L1Event>,
    l2_events: HeapProd<L2Event>,
    l3_events: HeapProd<L3Event>,
    trade_events: HeapProd<TradeEvent>,
    client_events: HeapProd<ClientEvent>,
}
impl SeparateEventFeeds {
    pub fn new(
        l1_events: HeapProd<L1Event>,
        l2_events: HeapProd<L2Event>,
        l3_events: HeapProd<L3Event>,
        trade_events: HeapProd<TradeEvent>,
        client_events: HeapProd<ClientEvent>,
    ) -> Self {
        Self {
            l1_events,
            l2_events,
            l3_events,
            trade_events,
            client_events,
        }
    }
}
impl EventSink for SeparateEventFeeds {
    /// Matches a market event to the correct feed based on type and pushes
    /// Blocks until able to be pushed into feed
    fn push(&mut self, _event: MarketEvent) {
        match _event.kind {
            MarketEventType::L1(event) => while self.l1_events.try_push(event).is_err() {},
            MarketEventType::L2(event) => while self.l2_events.try_push(event).is_err() {},
            MarketEventType::L3(event) => while self.l3_events.try_push(event).is_err() {},
            MarketEventType::Trade(event) => while self.trade_events.try_push(event).is_err() {},
            MarketEventType::Client(event) => while self.client_events.try_push(event).is_err() {},
        }
    }
}

/// Event feed struct that drops all events pushed into it
pub struct NullFeeds {}
impl EventSink for NullFeeds {
    fn push(&mut self, _event: MarketEvent) {}
}

/// Event feed struct with a single unified queue for pushing events into
/// Specific event types should be parsed out later in the pipeline
pub struct SingleEventFeed {
    events: HeapProd<MarketEvent>,
}
impl SingleEventFeed {
    pub fn new(events: HeapProd<MarketEvent>) -> Self {
        Self { events }
    }
}
impl EventSink for SingleEventFeed {
    fn push(&mut self, _event: MarketEvent) {
        while self.events.try_push(_event).is_err() {}
    }
}
