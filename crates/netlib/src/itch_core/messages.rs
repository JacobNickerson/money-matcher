use std::str::from_utf8;

use crate::itch_core::{helpers::encode_u48, types::*};
use zerocopy::byteorder::{U16, U32, U64};
pub trait ItchMessage {
    fn set_tracking_number(&mut self, n: u16);
    fn set_stock_locate(&mut self, n: u16);
}

impl OrderExecutedMessage {
    const MESSAGE_TYPE: u8 = b'E';

    pub fn new(
        stock_locate: u16,
        timestamp: u64,
        order_reference_number: u64,
        executed_shares: u32,
        match_number: u64,
    ) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            stock_locate: U16::new(stock_locate),
            tracking_number: 0.into(),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            executed_shares: U32::new(executed_shares),
            match_number: U64::new(match_number),
        }
    }
}

impl ItchMessage for OrderExecutedMessage {
    fn set_tracking_number(&mut self, n: u16) {
        self.tracking_number = U16::new(n);
    }

    fn set_stock_locate(&mut self, n: u16) {
        self.stock_locate = U16::new(n);
    }
}

impl AddOrder {
    const MESSAGE_TYPE: u8 = b'A';

    pub fn new(
        stock_locate: u16,
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
            tracking_number: 0.into(),
            timestamp: encode_u48(timestamp),
            order_reference_number: U64::new(order_reference_number),
            buy_sell_indicator,
            shares: U32::new(shares),
            stock,
            price: U32::new(price),
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: AddOrder | stock_locate={} | tracking_number={} | timestamp={:?} | order_reference_number={} | buy_sell_indicator={} | shares={} | stock={} | price={}",
            self.stock_locate.get(),
            self.tracking_number.get(),
            self.timestamp,
            self.order_reference_number.get(),
            self.buy_sell_indicator as char,
            self.shares.get(),
            from_utf8(&self.stock).expect("err"),
            self.price.get(),
        );
    }
}

impl ItchMessage for AddOrder {
    fn set_tracking_number(&mut self, n: u16) {
        self.tracking_number = U16::new(n);
    }

    fn set_stock_locate(&mut self, n: u16) {
        self.stock_locate = U16::new(n);
    }
}

impl TestBenchmark {
    const MESSAGE_TYPE: u8 = b'b';

    pub fn new(timestamp: u64) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            timestamp: encode_u48(timestamp),
            tracking_number: 0.into(),
            stock_locate: 0.into(),
        }
    }
}

impl ItchMessage for TestBenchmark {
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
    use crate::itch_core::helpers::encode_u48;

    #[test]
    fn test_order_executed_message_new_and_setters() {
        let mut msg = OrderExecutedMessage::new(1, 12, 123, 1234, 12345);

        assert_eq!(msg.message_type, b'E');
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.timestamp, encode_u48(12));
        assert_eq!(msg.order_reference_number.get(), 123);
        assert_eq!(msg.executed_shares.get(), 1234);
        assert_eq!(msg.match_number.get(), 12345);

        msg.set_tracking_number(12);
        msg.set_stock_locate(123);

        assert_eq!(msg.tracking_number.get(), 12);
        assert_eq!(msg.stock_locate.get(), 123);
    }

    #[test]
    fn test_add_order_new_and_setters() {
        let stock = *b"STOCK   ";
        let mut msg = AddOrder::new(1, 12, 123, b'B', 1234, stock, 12345);

        assert_eq!(msg.message_type, b'A');
        assert_eq!(msg.stock_locate.get(), 1);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.timestamp, encode_u48(12));
        assert_eq!(msg.order_reference_number.get(), 123);
        assert_eq!(msg.buy_sell_indicator, b'B');
        assert_eq!(msg.shares.get(), 1234);
        assert_eq!(&msg.stock, b"STOCK   ");
        assert_eq!(msg.price.get(), 12345);

        msg.set_tracking_number(12);
        msg.set_stock_locate(123);

        assert_eq!(msg.tracking_number.get(), 12);
        assert_eq!(msg.stock_locate.get(), 123);
    }
}
