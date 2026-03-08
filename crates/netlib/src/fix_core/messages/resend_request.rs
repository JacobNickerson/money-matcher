use crate::fix_core::messages::{
    FIX_MESSAGE_TYPE_RESEND_REQUEST, FixMessage, TAG_BEGIN_SEQ_NO, TAG_END_SEQ_NO,
};

/// The resend request is sent by the receiving application to initiate the retransmission of messages.
///
/// MsgType = 2
#[derive(Debug, Clone)]
pub struct ResendRequest {
    pub begin_seq_no: u32,
    pub end_seq_no: u32,
}

impl FixMessage for ResendRequest {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_RESEND_REQUEST;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(64);

        // 7 - BeginSeqNo
        buf.extend_from_slice(TAG_BEGIN_SEQ_NO);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.begin_seq_no).as_bytes());
        buf.push(0x01);

        // 16 - EndSeqNo
        buf.extend_from_slice(TAG_END_SEQ_NO);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.end_seq_no).as_bytes());
        buf.push(0x01);

        buf
    }
}
