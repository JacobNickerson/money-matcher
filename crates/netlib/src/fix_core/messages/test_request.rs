use crate::fix_core::messages::{FIX_MESSAGE_TYPE_TEST_REQUEST, FixMessage, TAG_TEST_REQ_ID};

/// TestRequest
/// The test request message forces a heartbeat from the opposing application.
///
/// MsgType = 1
#[derive(Debug)]
pub struct TestRequest {
    pub test_req_id: u32,
}

impl TestRequest {
    pub fn new(test_req_id: u32) -> Self {
        Self { test_req_id }
    }
}

impl FixMessage for TestRequest {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_TEST_REQUEST;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        buf.extend_from_slice(TAG_TEST_REQ_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.test_req_id).as_bytes());
        buf.push(0x01);

        buf
    }
}
