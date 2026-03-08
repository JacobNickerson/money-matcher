use crate::lob::order::Order;
use mio::Token;
use netlib::fix_core::messages::{
    FixMessage, execution_report::ExecutionReport, heartbeat::Heartbeat, logon::Logon,
    resend_request::ResendRequest, test_request::TestRequest,
};

pub mod engine;
pub mod session;

#[derive(Debug, Clone)]
pub struct FIXRequest {
    pub comp_id: String,
    pub message: FIXRequestMessage,
}

#[derive(Debug, Clone)]
pub enum FIXRequestMessage {
    Order(Order),
    Logon(Logon),
    Heartbeat(Heartbeat),
    TestRequest(TestRequest),
    ResendRequest(ResendRequest),
}
