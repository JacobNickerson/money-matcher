#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
	Buy,
	Sell,
	Bid,
	Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
	Active,
	Canceled,
}

pub type OrderId = u64;
pub type Price = u64;
pub type Timestamp = u64;