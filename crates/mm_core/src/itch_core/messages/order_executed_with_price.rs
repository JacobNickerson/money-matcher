use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE;

/// This message is sent whenever an order on the book is executed in whole or in part at a price different from the initial display price.
///
/// Since the execution price is different than the display price of the original Add Order, Nasdaq includes a price field within this execution message.
/// It is possible to receive multiple Order Executed and Order Executed With Price messages for the same order if that order is executed in several parts.
/// Multiple Order Executed messages on the same order are cumulative.
/// Executions may be marked as non-printable.
/// If the execution is marked as non-printed, it means the shares will be included into a later bulk print (e.g., in the case of cross executions).
/// If a firm is looking to use the data in time-and-sales displays or volume calculations, Nasdaq recommends that firms ignore messages marked as non-printable to prevent double counting.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderExecutedWithPrice {}

impl OrderExecutedWithPrice {
    /// Encodes an OrderExecutedWithPrice message directly into a provided byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The destination byte slice (must be at least 36 bytes)
    /// * `stock_locate` - Locate code identifying the security
    /// * `tracking_number` - Nasdaq internal tracking number
    /// * `timestamp` - Nanoseconds since midnight
    /// * `order_reference_number` - The unique reference number assigned to the new order at the time of receipt
    /// * `executed_shares` - The number of shares executed
    /// * `match_number` - The Nasdaq generated day unique Match Number of this execution
    /// * `printable` - Indicates if the execution should be reflected on time and sales displays: "N" = Non-Printable, "Y" = Printable
    /// * `execution_price` - The Price at which the order execution occurred
    #[inline(always)]
    pub fn encode_into(
        buf: &mut [u8],
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
        printable: u8,
        execution_price: u32,
    ) {
        buf[0] = ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE;
        buf[1..3].copy_from_slice(&stock_locate.to_be_bytes());
        buf[3..5].copy_from_slice(&tracking_number.to_be_bytes());
        buf[5..11].copy_from_slice(&encode_u48(timestamp));
        buf[11..19].copy_from_slice(&order_reference_number.to_be_bytes());
        buf[19..23].copy_from_slice(&executed_shares.to_be_bytes());
        buf[23..31].copy_from_slice(&match_number.to_be_bytes());
        buf[31] = printable;
        buf[32..36].copy_from_slice(&execution_price.to_be_bytes());
    }
}
