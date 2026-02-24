use crate::lob::types::{OrderId, Price, Timestamp};

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
    pub fn new(order: Order, qty: u64, price: Price) -> Self {
        Self {
            order_id: order.order_id,
            side: order.side,
            status: Self::DEFAULT_STATUS,
            qty,
            price,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Active,
    Canceled,
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
