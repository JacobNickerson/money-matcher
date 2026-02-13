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

    pub fn from_bytes(mut bytes: MessageData) -> Result<Packet, &'static str> {
        let header_bytes = bytes.split_to(20);
        let header = Header::read_from_prefix(&header_bytes).unwrap().0;

        let mc = u16::from_be_bytes(header.message_count) as usize;
        let mut message_blocks = Vec::with_capacity(mc);

        for _ in 0..mc {
            if bytes.len() < 2 {
                return Err("Err");
            }

            let len_bytes = bytes.split_to(2);
            let mut message_length: MessageLength = [0u8; 2];
            message_length.copy_from_slice(&len_bytes);

            let ml = u16::from_be_bytes(message_length) as usize;

            if bytes.len() < ml {
                return Err("Err");
            }

            let message_data = bytes.split_to(ml);

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
