use crate::MessageBlock;

pub type Socket = tokio::net::UdpSocket;
pub type SessionID = [u8; 10];
pub type SequenceNumber = [u8; 8];
pub type MessageCount = [u8; 2];
pub type MessageLength = [u8; 2];
pub type MessageData = Vec<u8>;
pub type Messages = Vec<MessageBlock>;
