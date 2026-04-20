use crate::lob_core::{ClientId, OrderId, OrderQty, Price, Timestamp};
use std::cmp::Ordering;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum OrderSide {
    Bid = b'B',
    Ask = b'S',
}

impl TryFrom<u8> for OrderSide {
    type Error = ();
    fn try_from(b: u8) -> Result<Self, Self::Error> {
        match b {
            b'B' => Ok(OrderSide::Bid),
            b'S' => Ok(OrderSide::Ask),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Active,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    pub client_id: ClientId,
    pub order_id: OrderId,
    pub side: OrderSide,
    pub timestamp: Timestamp,
    pub kind: OrderType,
}

impl Order {
    #[inline(always)]
    pub fn new(
        client_id: ClientId,
        order_id: OrderId,
        side: OrderSide,
        timestamp: Timestamp,
        kind: OrderType,
    ) -> Self {
        Self {
            client_id,
            order_id,
            side,
            timestamp,
            kind,
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.order_id.cmp(&other.order_id))
    }
}
impl Default for Order {
    fn default() -> Order {
        Order {
            client_id: 0,
            order_id: 0,
            side: OrderSide::Ask,
            timestamp: 0,
            kind: OrderType::Cancel { old_id: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit {
        qty: OrderQty,
        price: Price,
    },
    Market {
        qty: OrderQty,
    },
    Update {
        old_id: OrderId,
        qty: OrderQty,
        price: Price,
    },
    Cancel {
        old_id: OrderId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitOrder {
    pub client_id: ClientId,
    pub order_id: OrderId,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub qty: OrderQty,
    pub price: Price,
}

impl LimitOrder {
    const DEFAULT_STATUS: OrderStatus = OrderStatus::Active;
    #[inline(always)]
    pub fn new(order: Order) -> Self {
        match order.kind {
            OrderType::Limit { qty, price } => Self {
                client_id: order.client_id,
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price,
            },
            OrderType::Market { qty } => Self {
                client_id: order.client_id,
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price: match order.side {
                    OrderSide::Ask => 0,
                    OrderSide::Bid => OrderQty::MAX,
                },
            },
            OrderType::Update {
                qty,
                old_id: _,
                price,
            } => Self {
                client_id: order.client_id,
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price,
            },
            OrderType::Cancel { .. } => {
                panic!("LimitOrder cannot be constructed from an Order representing a cancel");
            }
        }
    }
}
