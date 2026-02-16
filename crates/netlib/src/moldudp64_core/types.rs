use bytes::Bytes;
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
