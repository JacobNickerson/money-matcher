use crate::fix_core::{
    iterator::FixIterator,
    messages::{FIX_MESSAGE_TYPE_TEST_REQUEST, FIXMessage, TAG_TEST_REQ_ID},
};
use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use std::str::from_utf8;

/// The test request message forces a heartbeat from the opposing application.
///
/// `MsgType = 1`
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct TestRequest {
    pub test_req_id: u32,
}

impl FIXMessage for TestRequest {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        buf.extend_from_slice(itoa_buf.format(TAG_TEST_REQ_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.test_req_id).as_bytes());
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut test_req_id = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_TEST_REQ_ID => {
                    test_req_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                _ => {}
            }
        }

        Ok(TestRequest {
            test_req_id: test_req_id.ok_or("Missing TestReqID")?,
        })
    }
}
