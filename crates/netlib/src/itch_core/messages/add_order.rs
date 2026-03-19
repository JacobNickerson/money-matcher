use crate::itch_core::helpers::{decode_u48, encode_u48};
use crate::itch_core::messages::{ITCH_MESSAGE_TYPE_ADD_ORDER, ItchMessage};
use std::str::from_utf8;
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// An Add Order Message indicates that a new order has been accepted by the Nasdaq system and was added to the displayable book.
///
/// The message includes a day-unique Order Reference Number used by Nasdaq to track the order.
/// Nasdaq supports two variations of the Add Order message format.
/// This message is generated for unattributed orders accepted by the Nasdaq system.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct AddOrder {
    /// Message Type "A" = Add Order - No MPID Attribution Message
    pub(crate) message_type: u8,
    /// Locate code identifying the security
    pub stock_locate: U16<BigEndian>,
    /// Nasdaq internal tracking number
    pub tracking_number: U16<BigEndian>,
    /// Nanoseconds since midnight
    pub timestamp: [u8; 6],
    /// The unique reference number assigned to the new order at the time of receipt
    pub order_reference_number: U64<BigEndian>,
    /// The type of order being added: "B" = Buy Order, "S" = Sell Order
    pub buy_sell_indicator: u8,
    /// The total number of shares associated with the order being added to the book
    pub shares: U32<BigEndian>,
    /// Stock symbol, right padded with spaces
    pub stock: [u8; 8],
    /// The display price of the new order
    pub price: U32<BigEndian>,
}

impl AddOrder {
    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        order_reference_number: u64,
        buy_sell_indicator: u8,
        shares: u32,
        stock: [u8; 8],
        price: u32,
    ) -> Self {
        Self {
            message_type: ITCH_MESSAGE_TYPE_ADD_ORDER,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            buy_sell_indicator,
            shares: U32::new(shares),
            stock,
            price: U32::new(price),
        }
    }

    pub fn encode_into(
        buf: &mut [u8],
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        buy_sell_indicator: u8,
        shares: u32,
        stock: [u8; 8],
        price: u32,
    ) {
        buf[0] = ITCH_MESSAGE_TYPE_ADD_ORDER;
        buf[1..3].copy_from_slice(&stock_locate.to_be_bytes());
        buf[3..5].copy_from_slice(&tracking_number.to_be_bytes());
        buf[5..11].copy_from_slice(&encode_u48(timestamp));
        buf[11..19].copy_from_slice(&order_reference_number.to_be_bytes());
        buf[19] = buy_sell_indicator;
        buf[20..24].copy_from_slice(&shares.to_be_bytes());
        buf[24..32].copy_from_slice(&stock);
        buf[32..36].copy_from_slice(&price.to_be_bytes());
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: AddOrder | stock_locate={} | tracking_number={} | timestamp={:?} | order_reference_number={} | buy_sell_indicator={} | shares={} | stock={} | price={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.order_reference_number.get(),
            self.buy_sell_indicator as char,
            self.shares.get(),
            from_utf8(&self.stock).expect("err"),
            self.price.get(),
        );
    }
}

impl ItchMessage for AddOrder {
    fn set_tracking_number(&mut self, n: u16) {
        self.tracking_number = U16::new(n);
    }

    fn set_stock_locate(&mut self, n: u16) {
        self.stock_locate = U16::new(n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_order_initial_state() {
        let stock_bytes = *b"STOCK   ";
        let price_val = 100;

        let msg = AddOrder::new(1, 1000, 5000, b'B', 10, stock_bytes, price_val);

        assert_eq!(msg.message_type, ITCH_MESSAGE_TYPE_ADD_ORDER);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.order_reference_number.get(), 5000);
        assert_eq!(msg.buy_sell_indicator, b'B');
        assert_eq!(msg.shares.get(), 10);
        assert_eq!(msg.stock, stock_bytes);
        assert_eq!(msg.price.get(), 1000000);

        msg.print();
    }

    #[test]
    fn test_add_order_trait_updates() {
        let mut msg = AddOrder::new(0, 0, 0, b'S', 0, *b"STOCK   ", 0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
