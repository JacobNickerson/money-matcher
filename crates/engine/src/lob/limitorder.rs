use crate::lob::types::{OrderId, Side, Price, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitOrder {
    pub order_id: OrderId,
    pub side: Side,
    pub price: Price,
    pub qty: u64,
	pub timestamp: Timestamp,
}

impl LimitOrder {
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
		debug_assert!(order_id > 0);
        Self {
            order_id,
            side,
            price,
            qty,
			timestamp,
        }
    }
}
