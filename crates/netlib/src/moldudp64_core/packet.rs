use zerocopy::{FromBytes, IntoBytes};

use crate::moldudp64_core::types::*;
impl Packet {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut capacity = 10 + 8 + 2;

        for message in &self.message_blocks {
            capacity += 2;
            capacity += message.message_data.len();
        }

        let mut bytes = Vec::with_capacity(capacity);

        bytes.extend_from_slice(self.header.as_bytes());

        for message in &self.message_blocks {
            bytes.extend_from_slice(&message.message_length);
            bytes.extend_from_slice(&message.message_data);
        }

        bytes
    }

    pub fn from_bytes(mut bytes: &[u8]) -> Result<Packet, &str> {
        let (header, remaining_bytes) = Header::read_from_prefix(bytes).unwrap();
        bytes = remaining_bytes;

        let mc = u16::from_be_bytes(header.message_count) as usize;
        let mut message_blocks = Vec::with_capacity(mc);

        for _ in 0..mc {
            if bytes.len() < 2 {
                return Err("Err");
            }

            let mut message_length: MessageLength = [0u8; 2];

            let (len_bytes, remaining_bytes) = bytes.split_at(2);
            message_length.copy_from_slice(len_bytes);
            let ml = u16::from_be_bytes(message_length) as usize;
            bytes = remaining_bytes;

            if bytes.len() < ml {
                return Err("Err");
            }

            let (msg_bytes, rest) = bytes.split_at(ml);
            let message_data = msg_bytes.to_vec();
            bytes = rest;

            message_blocks.push(MessageBlock {
                message_length,
                message_data,
            });
        }

        Ok(Packet {
            header,
            message_blocks,
        })
    }
}
