use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::ITCH_MESSAGE_TYPE_ORDER_REPLACE;

/// This message is sent whenever an order on the book has been cancel-replaced.
///
/// All remaining shares from the original order are no longer accessible and must be removed.
/// New order details are provided for the replacement, along with a new order reference number which will be used henceforth.
/// Since the side, stock symbol, and attribution (if any) cannot be changed by an Order Replace event, these fields are not included in the message.
/// Firms should retain the side, stock symbol, and MPID from the original Add Order message.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderReplace {}

impl OrderReplace {
    /// Encodes an OrderReplace message directly into a provided byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The destination byte slice (must be at least 35 bytes)
    /// * `stock_locate` - Locate code identifying the security
    /// * `tracking_number` - Nasdaq internal tracking number
    /// * `timestamp` - Nanoseconds since midnight
    /// * `original_order_reference_number` - The original order reference number of the order being replaced
    /// * `new_order_reference_number` - The new reference number for this order at time of replacement
    /// * `shares` - The new total displayed quantity
    /// * `price` - The new display price for the order
    pub fn encode_into(
        buf: &mut [u8],
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        original_order_reference_number: u64,
        new_order_reference_number: u64,
        shares: u32,
        price: u32,
    ) {
        buf[0] = ITCH_MESSAGE_TYPE_ORDER_REPLACE;
        buf[1..3].copy_from_slice(&stock_locate.to_be_bytes());
        buf[3..5].copy_from_slice(&tracking_number.to_be_bytes());
        buf[5..11].copy_from_slice(&encode_u48(timestamp));
        buf[11..19].copy_from_slice(&original_order_reference_number.to_be_bytes());
        buf[19..27].copy_from_slice(&new_order_reference_number.to_be_bytes());
        buf[27..31].copy_from_slice(&shares.to_be_bytes());
        buf[31..35].copy_from_slice(&price.to_be_bytes());
    }
}
