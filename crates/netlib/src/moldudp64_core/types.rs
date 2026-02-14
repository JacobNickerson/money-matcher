use bytes::Bytes;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub type Socket = tokio::net::UdpSocket;
pub type SessionID = [u8; 10];
pub type SequenceNumber = [u8; 8];
pub type MessageCount = [u8; 2];
pub type MessageLength = [u8; 2];
pub type MessageData = Bytes;
pub type MessageBlocks = Vec<MessageBlock>;

#[repr(C)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct Header {
    pub session_id: SessionID,
    pub sequence_number: SequenceNumber,
    pub message_count: MessageCount,
}

pub struct MessageBlock {
    pub message_length: MessageLength,
    pub message_data: MessageData,
}

pub struct Packet {
    pub header: Header,
    pub message_blocks: MessageBlocks,
}

pub struct RequestPacket {
    pub session_id: SessionID,
    pub sequence_number: SequenceNumber,
    pub message_count: MessageCount,
}
