use crate::lob_core::{ClientId, OrderId, OrderQty, Price, Timestamp};
use rkyv::{Archive, Deserialize, Serialize};
use std::cmp::Ordering;

/// Enum denoting the side of the order book an order belongs to
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]

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

/// Enum determining the current status of a limit order. Only used for limit orders
/// found in the order book
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Active,
    Canceled,
}

/// Typedef of the fixed-size array of bytes used for a serialized order
pub type OrderByteArray = [u8; size_of::<Order>()];

/// The Most Important Struct. Represents an order, with type specific information held in its OrderType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
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

    /// Serialize an order to a constant size byte array
    pub fn to_bytes(&self) -> OrderByteArray {
        let mut buf: OrderByteArray = [0u8; size_of::<Order>()];

        const CLID_START: usize = 0;
        const CLID_END: usize = size_of::<ClientId>();
        buf[CLID_START..CLID_END].copy_from_slice(&self.client_id.to_le_bytes());

        const ORDID_START: usize = CLID_END;
        const ORDID_END: usize = ORDID_START + size_of::<OrderId>();
        buf[ORDID_START..ORDID_END].copy_from_slice(&self.order_id.to_le_bytes());

        const SIDE_START: usize = ORDID_END;
        const SIDE_END: usize = SIDE_START + size_of::<OrderSide>();
        buf[SIDE_START] = match self.side {
            OrderSide::Ask => b'B',
            OrderSide::Bid => b'S',
        };
        const TIME_START: usize = SIDE_END;
        const TIME_END: usize = TIME_START + size_of::<Timestamp>();
        buf[TIME_START..TIME_END].copy_from_slice(&self.timestamp.to_le_bytes());

        const KIND_START: usize = TIME_END;
        const KIND_END: usize = KIND_START + size_of::<OrderType>();
        let mut kind = [0u8; size_of::<OrderType>()];
        buf[KIND_START..KIND_END].copy_from_slice(match self.kind {
            OrderType::Limit { qty, price } => {
                kind[0] = 1;

                const QTY_START: usize = 1;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                kind[QTY_START..QTY_END].copy_from_slice(&qty.to_le_bytes());

                const PRICE_START: usize = QTY_END;
                const PRICE_END: usize = PRICE_START + size_of::<Price>();
                kind[PRICE_START..PRICE_END].copy_from_slice(&price.to_le_bytes());

                &kind
            }
            OrderType::Market { qty } => {
                kind[0] = 2;

                const QTY_START: usize = 1;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                kind[QTY_START..QTY_END].copy_from_slice(&qty.to_le_bytes());

                &kind
            }
            OrderType::Update { old_id, qty, price } => {
                kind[0] = 3;

                const OLDID_START: usize = 1;
                const OLDID_END: usize = OLDID_START + size_of::<OrderId>();
                kind[OLDID_START..OLDID_END].copy_from_slice(&old_id.to_le_bytes());

                const QTY_START: usize = OLDID_END;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                kind[QTY_START..QTY_END].copy_from_slice(&qty.to_le_bytes());

                const PRICE_START: usize = QTY_END;
                const PRICE_END: usize = PRICE_START + size_of::<Price>();
                kind[PRICE_START..PRICE_END].copy_from_slice(&price.to_le_bytes());

                &kind
            }
            OrderType::Cancel { old_id } => {
                kind[0] = 4;

                const OLDID_START: usize = 1;
                const OLDID_END: usize = OLDID_START + size_of::<OrderId>();
                kind[OLDID_START..OLDID_END].copy_from_slice(&old_id.to_le_bytes());

                &kind
            }
        });
        buf
    }

    /// Construct an order from a constant size byte array
    pub fn from_bytes(buf: OrderByteArray) -> Self {
        const CLID_START: usize = 0;
        const CLID_END: usize = size_of::<ClientId>();
        let client_id = ClientId::from_le_bytes(buf[CLID_START..CLID_END].try_into().unwrap());

        const ORDID_START: usize = CLID_END;
        const ORDID_END: usize = ORDID_START + size_of::<OrderId>();
        let order_id = OrderId::from_le_bytes(buf[ORDID_START..ORDID_END].try_into().unwrap());

        const SIDE_START: usize = ORDID_END;
        const SIDE_END: usize = SIDE_START + size_of::<OrderSide>();
        let side = match buf[SIDE_START] {
            b'B' => OrderSide::Ask,
            b'S' => OrderSide::Bid,
            _ => panic!("error: attempted to deserialize an unknown side tag"),
        };

        const TIME_START: usize = SIDE_END;
        const TIME_END: usize = TIME_START + size_of::<Timestamp>();
        let timestamp = OrderId::from_le_bytes(buf[TIME_START..TIME_END].try_into().unwrap());

        const KIND_START: usize = TIME_END;
        let kind = match buf[KIND_START] {
            1 => {
                const QTY_START: usize = KIND_START + 1;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                const PRICE_START: usize = QTY_END;
                const PRICE_END: usize = PRICE_START + size_of::<Price>();
                OrderType::Limit {
                    qty: OrderQty::from_le_bytes(buf[QTY_START..QTY_END].try_into().unwrap()),
                    price: Price::from_le_bytes(buf[PRICE_START..PRICE_END].try_into().unwrap()),
                }
            }
            2 => {
                const QTY_START: usize = KIND_START + 1;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                OrderType::Market {
                    qty: OrderQty::from_le_bytes(buf[QTY_START..QTY_END].try_into().unwrap()),
                }
            }
            3 => {
                const OLDID_START: usize = KIND_START + 1;
                const OLDID_END: usize = OLDID_START + size_of::<OrderId>();
                const QTY_START: usize = OLDID_END;
                const QTY_END: usize = QTY_START + size_of::<OrderQty>();
                const PRICE_START: usize = QTY_END;
                const PRICE_END: usize = PRICE_START + size_of::<Price>();
                OrderType::Update {
                    old_id: OrderId::from_le_bytes(buf[OLDID_START..OLDID_END].try_into().unwrap()),
                    qty: OrderQty::from_le_bytes(buf[QTY_START..QTY_END].try_into().unwrap()),
                    price: Price::from_le_bytes(buf[PRICE_START..PRICE_END].try_into().unwrap()),
                }
            }
            4 => {
                const OLDID_START: usize = KIND_START + 1;
                const OLDID_END: usize = OLDID_START + size_of::<OrderId>();
                OrderType::Cancel {
                    old_id: OrderId::from_le_bytes(buf[OLDID_START..OLDID_END].try_into().unwrap()),
                }
            }
            _ => panic!("error: attempted to deserialize an unknown OrderType"),
        };
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

/// Enum containing type-specific information for an Order. Currently an Order can be either a
/// limit order, market order, update, or cancel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
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

/// Stripped down version of Order only used for Orders with type Limit. Used specifically for
/// storage inside of the limit order book
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

mod test {
    use super::*;

    #[test]
    fn test_limit_serialization() {
        let order = Order::new(
            61268,
            5819515,
            OrderSide::Ask,
            352895656,
            OrderType::Limit {
                qty: 357826,
                price: 9659,
            },
        );

        let bytes = order.to_bytes();
        let deserialized = Order::from_bytes(bytes);

        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_market_serialization() {
        let order = Order::new(
            61268,
            5819515,
            OrderSide::Bid,
            352895656,
            OrderType::Market { qty: 357826 },
        );

        let bytes = order.to_bytes();
        let deserialized = Order::from_bytes(bytes);

        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_update_serialization() {
        let order = Order::new(
            61268,
            5819515,
            OrderSide::Bid,
            352895656,
            OrderType::Update {
                old_id: 3855,
                qty: 0,
                price: Price::MAX,
            },
        );

        let bytes = order.to_bytes();
        let deserialized = Order::from_bytes(bytes);

        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_cancel_serialization() {
        let order = Order::new(
            61268,
            5819515,
            OrderSide::Ask,
            352895656,
            OrderType::Cancel { old_id: 99 },
        );

        let bytes = order.to_bytes();
        let deserialized = Order::from_bytes(bytes);

        assert_eq!(order, deserialized);
    }
}
