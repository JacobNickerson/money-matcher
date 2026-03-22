use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::ITCH_MESSAGE_TYPE_ORDER_EXECUTED;

/// This message is sent whenever an order on the book is executed in whole or in part.
///
/// It is possible to receive several Order Executed Messages for the same order reference number if that order is executed in several parts.
/// Multiple Order Executed Messages on the same order are cumulative.
/// By combining the executions from both types of Order Executed Messages and the Trade Message, it is possible to build a complete view of all non-cross executions that happen on Nasdaq.
/// Cross execution information is available in one bulk print per symbol via the Cross Trade Message.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderExecuted {}

impl OrderExecuted {
    /// Encodes an OrderExecuted message directly into a provided byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The destination byte slice (must be at least 31 bytes)
    /// * `stock_locate` - Locate code identifying the security
    /// * `tracking_number` - Nasdaq internal tracking number
    /// * `timestamp` - Nanoseconds since midnight
    /// * `order_reference_number` - The unique reference number assigned to the new order at the time of receipt
    /// * `executed_shares` - The number of shares executed
    /// * `match_number` - The Nasdaq generated day unique Match Number of this execution
    pub fn encode_into(
        buf: &mut [u8],
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
    ) {
        buf[0] = ITCH_MESSAGE_TYPE_ORDER_EXECUTED;
        buf[1..3].copy_from_slice(&stock_locate.to_be_bytes());
        buf[3..5].copy_from_slice(&tracking_number.to_be_bytes());
        buf[5..11].copy_from_slice(&encode_u48(timestamp));
        buf[11..19].copy_from_slice(&order_reference_number.to_be_bytes());
        buf[19..23].copy_from_slice(&executed_shares.to_be_bytes());
        buf[23..31].copy_from_slice(&match_number.to_be_bytes());
    }
}
