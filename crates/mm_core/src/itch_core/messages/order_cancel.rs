use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::ITCH_MESSAGE_TYPE_ORDER_CANCEL;

/// This message is sent whenever an order on the book is modified as a result of a partial cancellation.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderCancel {}

impl OrderCancel {
    /// Encodes an OrderCancel message directly into a provided byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The destination byte slice (must be at least 23 bytes)
    /// * `stock_locate` - Locate code identifying the security
    /// * `tracking_number` - Nasdaq internal tracking number
    /// * `timestamp` - Nanoseconds since midnight
    /// * `order_reference_number` - The reference number of the order being canceled
    /// * `canceled_shares` - The number of shares being removed from the display size of the order as a result of a cancellation
    #[inline(always)]
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
}
