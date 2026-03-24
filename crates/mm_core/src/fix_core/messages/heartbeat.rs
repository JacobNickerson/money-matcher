use crate::fix_core::{
    iterator::FixIterator,
    messages::{FIXMessage, TAG_TEST_REQ_ID},
};
use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use std::str::from_utf8;

/// During periods of message inactivity, FIX applications will generate Heartbeat messages at regular time intervals.
///
/// `MsgType = 0`
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub test_req_id: Option<u32>,
}

impl FIXMessage for Heartbeat {
    fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        if let Some(test_req_id) = self.test_req_id {
            let mut itoa_buf = itoa::Buffer::new();

            buf.extend_from_slice(itoa_buf.format(TAG_TEST_REQ_ID).as_bytes());
            buf.push(b'=');
            buf.extend_from_slice(itoa_buf.format(test_req_id).as_bytes());
            buf.push(0x01);
        }

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut test_req_id: Option<u32> = None;

        for (tag, value) in FixIterator::new(msg) {
            if tag == TAG_TEST_REQ_ID {
                test_req_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
            }
        }

        Ok(Self { test_req_id })
    }
}
