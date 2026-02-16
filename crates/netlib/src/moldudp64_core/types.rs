use bytes::Bytes;
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub type MessageBlocks = Vec<MessageBlock>;
pub type MessageCount = [u8; 2];
pub type MessageData = Bytes;
pub type MessageLength = [u8; 2];
pub type SequenceNumber = [u8; 8];
pub type SessionID = [u8; 10];
pub type Socket = std::net::UdpSocket;
pub type Event = Bytes;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct Header {
    pub session_id: SessionID,
    pub sequence_number: SequenceNumber,
    pub message_count: MessageCount,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBlock {
    pub message_data: MessageData,
    pub message_length: MessageLength,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub header: Header,
    pub message_blocks: MessageBlocks,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestPacket {
    pub message_count: MessageCount,
    pub sequence_number: SequenceNumber,
    pub session_id: SessionID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequencedEvent {
    pub event: Event,
    pub sequence_number: u64,
    pub session_id: [u8; 10],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct TestBenchmark {
    message_type: u8,
    pub timestamp: [u8; 6],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct OrderExecutedMessage {
    message_type: u8,
    pub stock_locate: U16<BigEndian>,
    pub tracking_number: U16<BigEndian>,
    pub timestamp: [u8; 6],
    pub order_reference_number: U64<BigEndian>,
    pub executed_shares: U32<BigEndian>,
    pub match_number: U64<BigEndian>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct AddOrder {
    pub message_type: u8,
    pub stock_locate: U16<BigEndian>,
    pub tracking_number: U16<BigEndian>,
    pub timestamp: [u8; 6],
    pub order_reference_number: U64<BigEndian>,
    pub buy_sell_indicator: u8,
    pub shares: U32<BigEndian>,
    pub stock: [u8; 8],
    pub price: U32<BigEndian>,
}

pub enum ItchEvent {
    TestBenchmark(TestBenchmark),
    AddOrder(AddOrder),
    OrderExecutedMessage(OrderExecutedMessage),
}

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
            timestamp: Self::encode_timestamp(timestamp),
            order_reference_number: U64::new(order_reference_number),
            executed_shares: U32::new(executed_shares),
            match_number: U64::new(match_number),
        }
    }

    fn encode_timestamp(value: u64) -> [u8; 6] {
        let bytes = value.to_be_bytes();
        let mut out = [0u8; 6];
        out.copy_from_slice(&bytes[2..]);
        out
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
            timestamp: Self::encode_timestamp(timestamp),
            order_reference_number: U64::new(order_reference_number),
            buy_sell_indicator,
            shares: U32::new(shares),
            stock,
            price: U32::new(price),
        }
    }

    fn encode_timestamp(value: u64) -> [u8; 6] {
        let bytes = value.to_be_bytes();
        let mut out = [0u8; 6];
        out.copy_from_slice(&bytes[2..]);
        out
    }

    pub fn decode_timestamp(ts: [u8; 6]) -> u64 {
        let mut buf = [0u8; 8];
        buf[2..].copy_from_slice(&ts);
        u64::from_be_bytes(buf)
    }
}

impl TestBenchmark {
    const MESSAGE_TYPE: u8 = b'b';

    pub fn new(timestamp: u64) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE,
            timestamp: Self::encode_timestamp(timestamp),
        }
    }

    fn encode_timestamp(value: u64) -> [u8; 6] {
        let bytes = value.to_be_bytes();
        let mut out = [0u8; 6];
        out.copy_from_slice(&bytes[2..]);
        out
    }

    pub fn decode_timestamp(ts: [u8; 6]) -> u64 {
        let mut buf = [0u8; 8];
        buf[2..].copy_from_slice(&ts);
        u64::from_be_bytes(buf)
    }
}
