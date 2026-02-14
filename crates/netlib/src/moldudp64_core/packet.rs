use zerocopy::{FromBytes, IntoBytes};

use crate::moldudp64_core::types::*;
impl Packet {
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
