use crate::lob_core::{OrderId, Price, Timestamp};
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
    pub order_id: OrderId,
    pub side: OrderSide,
    pub timestamp: Timestamp,
    pub kind: OrderType,
}

impl Order {
    #[inline(always)]
    pub fn new(order_id: OrderId, side: OrderSide, timestamp: Timestamp, kind: OrderType) -> Self {
        Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit {
        qty: u64,
        price: Price,
    },
    Market {
        qty: u64,
    },
    Update {
        old_id: OrderId,
        qty: u64,
        price: Price,
    },
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitOrder {
    pub order_id: OrderId,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub qty: u64,
    pub price: Price,
}

impl LimitOrder {
    const DEFAULT_STATUS: OrderStatus = OrderStatus::Active;
    #[inline(always)]
    pub fn new(order: Order) -> Self {
        match order.kind {
            OrderType::Limit { qty, price } => Self {
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price,
            },
            OrderType::Market { qty } => Self {
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price: match order.side {
                    OrderSide::Ask => 0,
                    OrderSide::Bid => u64::MAX,
                },
            },
            OrderType::Update {
                qty,
                old_id: _,
                price,
            } => Self {
                order_id: order.order_id,
                side: order.side,
                status: Self::DEFAULT_STATUS,
                qty,
                price,
            },
            OrderType::Cancel => {
                panic!("LimitOrder cannot be constructed from an Order representing a cancel");
            }
        }
    }
}
