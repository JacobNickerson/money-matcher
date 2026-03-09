use std::str::from_utf8;

use crate::fix_core::{
    iterator::FixIterator,
    messages::{FIX_MESSAGE_TYPE_RESEND_REQUEST, FIXMessage, TAG_BEGIN_SEQ_NO, TAG_END_SEQ_NO},
};

/// The resend request is sent by the receiving application to initiate the retransmission of messages.
///
/// `MsgType = 2`
#[derive(Debug, Clone)]
pub struct ResendRequest {
    pub begin_seq_no: u32,
    pub end_seq_no: u32,
}

impl FIXMessage for ResendRequest {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_RESEND_REQUEST;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(64);

        buf.extend_from_slice(TAG_BEGIN_SEQ_NO);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.begin_seq_no).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_END_SEQ_NO);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.end_seq_no).as_bytes());
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut begin_seq_no: Option<u32> = None;
        let mut end_seq_no: Option<u32> = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_BEGIN_SEQ_NO => {
                    begin_seq_no = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_END_SEQ_NO => {
                    end_seq_no = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                _ => {}
            }
        }

        Ok(ResendRequest {
            begin_seq_no: begin_seq_no.ok_or("Missing BeginSeqNo")?,
            end_seq_no: end_seq_no.ok_or("Missing EndSeqNo")?,
        })
    }
}
