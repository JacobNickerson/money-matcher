use crate::lob::order::Order;
use mio::Token;
use netlib::fix_core::messages::{
    FixMessage, execution_report::ExecutionReport, heartbeat::Heartbeat, logon::Logon,
    test_request::TestRequest,
};

pub mod engine;
pub mod session;

#[derive(Debug)]
pub struct FIXRequest {
    pub comp_id: String,
    pub message: FIXRequestMessage,
}

#[derive(Debug)]
pub enum FIXRequestMessage {
    Order(Order),
    Logon(Logon),
    Heartbeat(Heartbeat),
    TestRequest(TestRequest),
}
