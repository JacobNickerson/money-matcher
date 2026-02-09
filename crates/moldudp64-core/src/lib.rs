mod sessions;
mod types;
use crate::types::{MessageCount, MessageData, MessageLength, Messages, SequenceNumber, SessionID};
use std::collections::HashMap;

pub struct MOLDUDP64 {
    pub socket: tokio::net::UdpSocket,
}

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
    pub messages: Messages,
}

impl Packet {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&self.header.session_id);
        bytes.extend_from_slice(&self.header.sequence_number);
        bytes.extend_from_slice(&self.header.message_count);

        for message in &self.messages {
            bytes.extend_from_slice(&message.message_length);
            bytes.extend_from_slice(&message.message_data);
        }

        bytes
    }

    pub fn from_bytes(mut bytes: &[u8]) -> Packet {
        let mut session_id = [0u8; 10];
        session_id.copy_from_slice(&bytes[..10]);
        bytes = &bytes[10..];

        let mut sequence_number = [0u8; 8];
        sequence_number.copy_from_slice(&bytes[..8]);
        bytes = &bytes[8..];

        let mut message_count = [0u8; 2];
        message_count.copy_from_slice(&bytes[..2]);
        bytes = &bytes[2..];

        let mut messages: Messages = Vec::new();

        let mc = u16::from_be_bytes(message_count) as usize;

        for _ in 0..mc {
            let mut message_length = [0u8; 2];
            message_length.copy_from_slice(&bytes[..2]);
            bytes = &bytes[2..];

            let ml = u16::from_be_bytes(message_length) as usize;

            let message_data = bytes[..ml].to_vec();
            bytes = &bytes[ml..];

            let block = MessageBlock {
                message_length,
                message_data,
            };

            messages.push(block);
        }

        let packet = Packet {
            header: Header {
                session_id,
                sequence_number,
                message_count,
            },
            messages,
        };

        packet
    }
}

pub struct RequestPacket {
    pub session_id: SessionID,
    pub sequence_number: SequenceNumber,
    pub message_count: MessageCount,
}

pub struct SessionTable {
    pub sessions: HashMap<SessionID, SequenceNumber>,
}

#[test]
fn test_message_block() {
    let message = "Hello, World!";
    let message_length: MessageLength = (message.len() as u16).to_be_bytes();
    let message_data: MessageData = message.as_bytes().to_vec();

    println!("Original Message: {:?}", message);
    println!("Message Length: {:?}", message_length);
    println!("Message Data: {:?}", message_data);

    let block = MessageBlock {
        message_length,
        message_data: message_data,
    };

    let reconstructed_length = u16::from_be_bytes(block.message_length);
    let reconstructed_data = &block.message_data;

    println!("Reconstructed Length: {:?}", reconstructed_length);
    println!(
        "Reconstructed Data: {:?}",
        std::str::from_utf8(reconstructed_data).unwrap()
    );
}
