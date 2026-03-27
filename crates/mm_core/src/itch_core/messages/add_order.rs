use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::ITCH_MESSAGE_TYPE_ADD_ORDER;

/// An Add Order Message indicates that a new order has been accepted by the Nasdaq system and was added to the displayable book.
///
/// The message includes a day-unique Order Reference Number used by Nasdaq to track the order.
/// Nasdaq supports two variations of the Add Order message format.
/// This message is generated for unattributed orders accepted by the Nasdaq system.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddOrder {}

impl AddOrder {
    /// Encodes an AddOrder message directly into a provided byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The destination byte slice (must be at least 36 bytes)
    /// * `stock_locate` - Locate code identifying the security
    /// * `tracking_number` - Nasdaq internal tracking number
    /// * `timestamp` - Nanoseconds since midnight
    /// * `order_reference_number` - The unique reference number assigned to the new order at the time of receipt
    /// * `buy_sell_indicator` - The type of order being added: "B" = Buy Order, "S" = Sell Order
    /// * `shares` - The total number of shares associated with the order being added to the book
    /// * `stock` - Stock symbol, right padded with spaces
    /// * `price` - The display price of the new order
    #[inline(always)]
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
}
