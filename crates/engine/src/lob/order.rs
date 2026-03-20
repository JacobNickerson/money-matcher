use core::lob_core::{
    OrderId, Price,
    market_orders::{Order, OrderSide, OrderStatus, OrderType},
};

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
            _ => {
                panic!("LimitOrder cannot be constructed from an Order representing a cancel");
            }
        }
    }
}
