use crate::fix_core::messages::{FIX_MESSAGE_TYPE_HEARTBEAT, FixMessage, TAG_TEST_REQ_ID};

/// Heartbeat
/// During periods of message inactivity, FIX applications will generate Heartbeat messages at regular time intervals.
///
/// MsgType = 0
#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub test_req_id: Option<u32>,
}

impl Heartbeat {
    pub fn new(test_req_id: Option<u32>) -> Self {
        Self { test_req_id }
    }
}

impl FixMessage for Heartbeat {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_HEARTBEAT;

    fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        if let Some(test_req_id) = self.test_req_id {
            let mut itoa_buf = itoa::Buffer::new();

            buf.extend_from_slice(TAG_TEST_REQ_ID);
            buf.push(b'=');
            buf.extend_from_slice(itoa_buf.format(test_req_id).as_bytes());
            buf.push(0x01);
        }

        buf
    }
}
