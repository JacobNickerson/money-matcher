use crate::itch_core::helpers::{decode_u48, encode_u48};
use crate::itch_core::messages::{ITCH_MESSAGE_TYPE_ORDER_EXECUTED, ItchMessage};
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// This message is sent whenever an order on the book is executed in whole or in part.
///
/// It is possible to receive several Order Executed Messages for the same order reference number if that order is executed in several parts.
/// Multiple Order Executed Messages on the same order are cumulative.
/// By combining the executions from both types of Order Executed Messages and the Trade Message, it is possible to build a complete view of all non-cross executions that happen on Nasdaq.
/// Cross execution information is available in one bulk print per symbol via the Cross Trade Message.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderExecuted {
    /// Message Type "E" = Order Executed Message
    pub(crate) message_type: u8,
    /// Locate code identifying the security
    pub stock_locate: U16<BigEndian>,
    /// Nasdaq internal tracking number
    pub tracking_number: U16<BigEndian>,
    /// Nanoseconds since midnight
    pub timestamp: [u8; 6],
    /// The unique reference number assigned to the new order at the time of receipt
    pub order_reference_number: U64<BigEndian>,
    /// The number of shares executed
    pub executed_shares: U32<BigEndian>,
    /// The Nasdaq generated day unique Match Number of this execution
    pub match_number: U64<BigEndian>,
}

impl OrderExecuted {
    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
    ) -> Self {
        Self {
            message_type: ITCH_MESSAGE_TYPE_ORDER_EXECUTED,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            executed_shares: U32::new(executed_shares),
            match_number: U64::new(match_number),
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: OrderExecuted | stock_locate={} | tracking_number={} | timestamp={:?} | order_ref={} | executed_shares={} | match_number={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.order_reference_number.get(),
            self.executed_shares.get(),
            self.match_number.get(),
        );
    }	
}

impl ItchMessage for OrderExecuted {
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
    fn test_order_executed_initial_state() {
        let msg = OrderExecuted::new(1, 1000, 5000, 100, 9999);

        assert_eq!(msg.message_type, ITCH_MESSAGE_TYPE_ORDER_EXECUTED);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.order_reference_number.get(), 5000);
        assert_eq!(msg.executed_shares.get(), 100);
        assert_eq!(msg.match_number.get(), 9999);

        msg.print();
    }

    #[test]
    fn test_order_executed_trait_updates() {
        let mut msg = OrderExecuted::new(0, 0, 0, 0, 0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
