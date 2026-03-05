use crate::itch_core::helpers::{decode_price, decode_u48, encode_price, encode_u48};
use crate::itch_core::messages::{ITCH_MESSAGE_TYPE_ORDER_REPLACE, ItchMessage};
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// This message is sent whenever an order on the book has been cancel-replaced.
///
/// All remaining shares from the original order are no longer accessible and must be removed.
/// New order details are provided for the replacement, along with a new order reference number which will be used henceforth.
/// Since the side, stock symbol, and attribution (if any) cannot be changed by an Order Replace event, these fields are not included in the message.
/// Firms should retain the side, stock symbol, and MPID from the original Add Order message.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderReplace {
    /// Message Type "U" = Order Replace Message
    pub(crate) message_type: u8,
    /// Locate code identifying the security
    pub stock_locate: U16<BigEndian>,
    /// Nasdaq internal tracking number
    pub tracking_number: U16<BigEndian>,
    /// Nanoseconds since midnight
    pub timestamp: [u8; 6],
    /// The original order reference number of the order being replaced
    pub original_order_reference_number: U64<BigEndian>,
    /// The new reference number for this order at time of replacement
    pub new_order_reference_number: U64<BigEndian>,
    /// The new total displayed quantity
    pub shares: U32<BigEndian>,
    /// The new display price for the order
    pub price: U32<BigEndian>,
}

impl OrderReplace {
    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        original_order_reference_number: u64,
        new_order_reference_number: u64,
        shares: u32,
        price: f64,
    ) -> Self {
        Self {
            message_type: ITCH_MESSAGE_TYPE_ORDER_REPLACE,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            original_order_reference_number: U64::new(original_order_reference_number),
            new_order_reference_number: U64::new(new_order_reference_number),
            shares: U32::new(shares),
            price: U32::new(encode_price(price)),
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: OrderReplace | stock_locate={} | tracking_number={} | timestamp={:?} | original_ref={} | new_ref={} | shares={} | price={:.4}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.original_order_reference_number.get(),
            self.new_order_reference_number.get(),
            self.shares.get(),
            decode_price(self.price.get()),
        );
    }
}

impl ItchMessage for OrderReplace {
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
    fn test_order_replace_initial_state() {
        let price_val = 100.0;
        let msg = OrderReplace::new(1, 1000, 5000, 5001, 20, price_val);

        assert_eq!(msg.message_type, ITCH_MESSAGE_TYPE_ORDER_REPLACE);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.original_order_reference_number.get(), 5000);
        assert_eq!(msg.new_order_reference_number.get(), 5001);
        assert_eq!(msg.shares.get(), 20);
        assert_eq!(msg.price.get(), 1000000);

        msg.print();
    }

    #[test]
    fn test_order_replace_trait_updates() {
        let mut msg = OrderReplace::new(0, 0, 0, 0, 0, 0.0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
