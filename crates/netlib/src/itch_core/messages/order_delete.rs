use crate::itch_core::helpers::{decode_u48, encode_u48};
use crate::itch_core::messages::{ItchMessage, MESSAGE_TYPE_ORDER_DELETE};
use zerocopy::byteorder::{BigEndian, U16, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// This message is sent whenever an order on the book is being cancelled.
///
/// All remaining shares are no longer accessible, so the order must be removed from the book.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderDelete {
    /// Message Type "D" = Order Delete Message
    pub(crate) message_type: u8,
    /// Locate code identifying the security
    pub stock_locate: U16<BigEndian>,
    /// Nasdaq internal tracking number
    pub tracking_number: U16<BigEndian>,
    /// Nanoseconds since midnight
    pub timestamp: [u8; 6],
    /// The reference number of the order being canceled
    pub order_reference_number: U64<BigEndian>,
}
impl OrderDelete {
    pub fn new(stock_locate: u16, timestamp: u64, order_reference_number: u64) -> Self {
        Self {
            message_type: MESSAGE_TYPE_ORDER_DELETE,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: OrderDelete | stock_locate={} | tracking_number={} | timestamp={:?} | order_ref={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.order_reference_number.get(),
        );
    }
}

impl ItchMessage for OrderDelete {
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
    fn test_order_delete_initial_state() {
        let msg = OrderDelete::new(1, 1000, 5000);

        assert_eq!(msg.message_type, MESSAGE_TYPE_ORDER_DELETE);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.order_reference_number.get(), 5000);

        msg.print();
    }

    #[test]
    fn test_order_delete_trait_updates() {
        let mut msg = OrderDelete::new(0, 0, 0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
