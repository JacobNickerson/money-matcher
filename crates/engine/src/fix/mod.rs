use crate::lob::order::Order;
use mio::Token;
use netlib::fix_core::messages::{FixMessage, execution_report::ExecutionReport, logon::Logon};

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
}

#[derive(Debug)]
pub struct FIXReply {
    pub comp_id: String,
    pub message: FIXReplyMessage,
}

#[derive(Debug)]
pub enum FIXReplyMessage {
    ExecutionReport(ExecutionReport),
    Logon(Logon),
}

impl FIXReplyMessage {
    pub fn message_type(&self) -> &'static [u8] {
        match self {
            FIXReplyMessage::ExecutionReport(_) => ExecutionReport::MESSAGE_TYPE,
            FIXReplyMessage::Logon(_) => Logon::MESSAGE_TYPE,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            FIXReplyMessage::ExecutionReport(er) => er.as_bytes(),
            FIXReplyMessage::Logon(l) => l.as_bytes(),
        }
    }
}
