use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct AddOrder {
    pub(crate) message_type: u8,
    pub stock_locate: U16<BigEndian>,
    pub tracking_number: U16<BigEndian>,
    pub timestamp: [u8; 6],
    pub order_reference_number: U64<BigEndian>,
    pub buy_sell_indicator: u8,
    pub shares: U32<BigEndian>,
    pub stock: [u8; 8],
    pub price: U32<BigEndian>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct TestBenchmark {
    pub(crate) message_type: u8,
    pub timestamp: [u8; 6],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderExecutedMessage {
    pub(crate) message_type: u8,
    pub stock_locate: U16<BigEndian>,
    pub tracking_number: U16<BigEndian>,
    pub timestamp: [u8; 6],
    pub order_reference_number: U64<BigEndian>,
    pub executed_shares: U32<BigEndian>,
    pub match_number: U64<BigEndian>,
}

pub enum ItchEvent {
    TestBenchmark(TestBenchmark),
    AddOrder(AddOrder),
    OrderExecutedMessage(OrderExecutedMessage),
}
