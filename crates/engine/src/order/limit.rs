#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
	Buy,
	Sell,
}

pub struct LimitOrder {
    pub order_id: u64,
    pub side: Side,
    pub price: u64,
    pub qty: u64,
	pub timestamp: u64,
}

impl LimitOrder {
    #[inline]
    pub fn new(
		order_id: u64,
		side: Side,
		price: u64,
		qty: u64,
		timestamp: u64,
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
