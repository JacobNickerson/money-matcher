use crate::itch_core::helpers::{decode_u48, encode_u48};
use crate::itch_core::messages::{ITCH_MESSAGE_TYPE_ORDER_CANCEL, ItchMessage};
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// This message is sent whenever an order on the book is modified as a result of a partial cancellation.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderCancel {
    /// Message Type "X" = Order Cancel Message
    pub(crate) message_type: u8,
    /// Locate code identifying the security
    pub stock_locate: U16<BigEndian>,
    /// Nasdaq internal tracking number
    pub tracking_number: U16<BigEndian>,
    /// Nanoseconds since midnight
    pub timestamp: [u8; 6],
    /// The reference number of the order being canceled
    pub order_reference_number: U64<BigEndian>,
    /// The number of shares being removed from the display size of the order as a result of a cancellation
    pub canceled_shares: U32<BigEndian>,
}

impl OrderCancel {
    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        order_reference_number: u64,
        canceled_shares: u32,
    ) -> Self {
        Self {
            message_type: ITCH_MESSAGE_TYPE_ORDER_CANCEL,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            canceled_shares: U32::new(canceled_shares),
        }
    }

    pub fn encode_into(
        buf: &mut [u8],
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        canceled_shares: u32,
    ) {
        buf[0] = ITCH_MESSAGE_TYPE_ORDER_CANCEL;
        buf[1..3].copy_from_slice(&stock_locate.to_be_bytes());
        buf[3..5].copy_from_slice(&tracking_number.to_be_bytes());
        buf[5..11].copy_from_slice(&encode_u48(timestamp));
        buf[11..19].copy_from_slice(&order_reference_number.to_be_bytes());
        buf[19..23].copy_from_slice(&canceled_shares.to_be_bytes());
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: OrderCancel | stock_locate={} | tracking_number={} | timestamp={:?} | order_ref={} | canceled_shares={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.order_reference_number.get(),
            self.canceled_shares.get(),
        );
    }
}

impl ItchMessage for OrderCancel {
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
    fn test_order_cancel_initial_state() {
        let msg = OrderCancel::new(1, 1000, 5000, 10);

        assert_eq!(msg.message_type, ITCH_MESSAGE_TYPE_ORDER_CANCEL);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.order_reference_number.get(), 5000);
        assert_eq!(msg.canceled_shares.get(), 10);

        msg.print();
    }

    #[test]
    fn test_order_cancel_trait_updates() {
        let mut msg = OrderCancel::new(0, 0, 0, 0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
