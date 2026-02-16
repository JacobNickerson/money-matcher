use crate::itch_core::{helpers::encode_u48, types::*};
use zerocopy::byteorder::{U16, U32, U64};

impl OrderExecutedMessage {
    const MESSAGE_TYPE: u8 = b'E';

    pub fn new(
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
    ) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(tracking_number),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            executed_shares: U32::new(executed_shares),
            match_number: U64::new(match_number),
        }
    }
}

impl AddOrder {
    const MESSAGE_TYPE: u8 = b'A';

    pub fn new(
        stock_locate: u16,
        tracking_number: u16,
        timestamp: u64,
        order_reference_number: u64,
        buy_sell_indicator: u8,
        shares: u32,
        stock: [u8; 8],
        price: u32,
    ) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            stock_locate: U16::new(stock_locate),
            tracking_number: U16::new(tracking_number),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            buy_sell_indicator,
            shares: U32::new(shares),
            stock,
            price: U32::new(price),
        }
    }
}

impl TestBenchmark {
    const MESSAGE_TYPE: u8 = b'b';

    pub fn new(timestamp: u64) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            timestamp: encode_u48(timestamp),
        }
    }
}
