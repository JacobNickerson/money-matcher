use crate::itch_core::helpers::{decode_price, decode_u48, encode_price, encode_u48};
use crate::itch_core::messages::{ItchMessage, MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE};
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// This message is sent whenever an order on the book is executed in whole or in part at a price different from the initial display price.
///
/// Since the execution price is different than the display price of the original Add Order, Nasdaq includes a price field within this execution message.
/// It is possible to receive multiple Order Executed and Order Executed With Price messages for the same order if that order is executed in several parts.
/// Multiple Order Executed messages on the same order are cumulative.
/// Executions may be marked as non-printable.
/// If the execution is marked as non-printed, it means the shares will be included into a later bulk print (e.g., in the case of cross executions).
/// If a firm is looking to use the data in time-and-sales displays or volume calculations, Nasdaq recommends that firms ignore messages marked as non-printable to prevent double counting.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderExecutedWithPrice {
    /// Message Type "C" = Order Executed with Price Message
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
    /// Indicates if the execution should be reflected on time and sales displays: "N" = Non-Printable, "Y" = Printable
    pub printable: u8,
    /// The Price at which the order execution occurred
    pub execution_price: U32<BigEndian>,
}

impl OrderExecutedWithPrice {
    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
        printable: u8,
        execution_price: f64,
    ) -> Self {
        Self {
            message_type: MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(0),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            executed_shares: U32::new(executed_shares),
            match_number: U64::new(match_number),
            printable,
            execution_price: U32::new(encode_price(execution_price)),
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: OrderExecutedWithPrice | stock_locate={} | tracking_number={} | timestamp={:?} | order_ref={} | shares={} | match={} | printable={} | price={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
            self.order_reference_number.get(),
            self.executed_shares.get(),
            self.match_number.get(),
            self.printable as char,
            decode_price(self.execution_price.get()),
        );
    }
}

impl ItchMessage for OrderExecutedWithPrice {
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
    fn test_order_executed_with_price_initial_state() {
        let price_val = 100.0;
        let msg = OrderExecutedWithPrice::new(1, 1000, 5000, 10, 9999, b'Y', price_val);

        assert_eq!(msg.message_type, MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE);
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.order_reference_number.get(), 5000);
        assert_eq!(msg.executed_shares.get(), 10);
        assert_eq!(msg.match_number.get(), 9999);
        assert_eq!(msg.printable, b'Y');
        assert_eq!(msg.execution_price.get(), 1000000);

        msg.print();
    }

    #[test]
    fn test_order_executed_with_price_trait_updates() {
        let mut msg = OrderExecutedWithPrice::new(0, 0, 0, 0, 0, b'N', 0.0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
