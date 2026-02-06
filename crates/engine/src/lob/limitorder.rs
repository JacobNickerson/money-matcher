use crate::lob::types::{OrderId, Side, OrderStatus, Price, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitOrder {
    pub order_id: OrderId,
    pub side: Side,
    pub status: OrderStatus,
    pub price: Price,
    pub qty: u64,
	pub timestamp: Timestamp,
}

impl LimitOrder {
    const DEFAULT_STATUS: OrderStatus = OrderStatus::Active;
    #[inline(always)]
    pub fn new(
		order_id: OrderId,
		side: Side,
		price: Price,
		qty: u64,
		timestamp: Timestamp,
    ) -> Self {
        debug_assert!(qty > 0);
        debug_assert!(price > 0);
        Self {
            order_id,
            side,
            status: Self::DEFAULT_STATUS,
            price,
            qty,
			timestamp,
        }
    }
}
