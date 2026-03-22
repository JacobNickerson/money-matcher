use crate::fix_core::messages::{
    execution_report::ExecutionReport, heartbeat::Heartbeat, logon::Logon,
    new_order_single::NewOrderSingle, order_cancel::OrderCancel,
    order_cancel_reject::OrderCancelReject, resend_request::ResendRequest,
    test_request::TestRequest,
};
use pyo3::{ffi::getter, pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pymethods};
use std::sync::Arc;

pub trait FIXMessage {
    const MESSAGE_TYPE: &'static [u8];
    fn as_bytes(&self) -> Vec<u8>;
    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str>
    where
        Self: Sized;
}

pub struct FixFrame {
    pub msg_type: &'static [u8],
    pub body: Vec<u8>,
}

pub mod execution_report;
pub mod heartbeat;
pub mod logon;
pub mod new_order_single;
pub mod order_cancel;
pub mod order_cancel_reject;
pub mod order_cancel_replace;
pub mod resend_request;
pub mod test_request;
pub mod types;

pub const FIX_MESSAGE_TYPE_HEARTBEAT: &[u8] = b"0";
pub const FIX_MESSAGE_TYPE_TEST_REQUEST: &[u8] = b"1";
pub const FIX_MESSAGE_TYPE_EXECUTION_REPORT: &[u8] = b"8";
pub const FIX_MESSAGE_TYPE_NEW_ORDER: &[u8] = b"D";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT: &[u8] = b"9";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE: &[u8] = b"G";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL: &[u8] = b"F";
pub const FIX_MESSAGE_TYPE_LOGON: &[u8] = b"A";
pub const FIX_MESSAGE_TYPE_RESEND_REQUEST: &[u8] = b"2";

pub const TAG_POSS_DUP_FLAG: &[u8] = b"43";
pub const TAG_BEGIN_SEQ_NO: &[u8] = b"7";
pub const TAG_END_SEQ_NO: &[u8] = b"16";
pub const TAG_BEGIN_STRING: &[u8] = b"8";
pub const TAG_BODY_LENGTH: &[u8] = b"9";
pub const TAG_CHECKSUM: &[u8] = b"10";
pub const TAG_CL_ORD_ID: &[u8] = b"11";
pub const TAG_CUM_QTY: &[u8] = b"14";
pub const TAG_EXEC_ID: &[u8] = b"17";
pub const TAG_EXEC_TRANS_TYPE: &[u8] = b"20";
pub const TAG_HANDL_INST: &[u8] = b"21";
pub const TAG_MSG_TYPE: &[u8] = b"35";
pub const TAG_ORDER_ID: &[u8] = b"37";
pub const TAG_ORDER_QTY: &[u8] = b"38";
pub const TAG_ORD_STATUS: &[u8] = b"39";
pub const TAG_ORD_TYPE: &[u8] = b"40";
pub const TAG_ORIG_CL_ORD_ID: &[u8] = b"41";
pub const TAG_PRICE: &[u8] = b"44";
pub const TAG_SECURITY_ID: &[u8] = b"48";
pub const TAG_SENDER_COMP_ID: &[u8] = b"49";
pub const TAG_SENDING_TIME: &[u8] = b"52";
pub const TAG_SIDE: &[u8] = b"54";
pub const TAG_SYMBOL: &[u8] = b"55";
pub const TAG_TARGET_COMP_ID: &[u8] = b"56";
pub const TAG_TEXT: &[u8] = b"58";
pub const TAG_TRANSACT_TIME: &[u8] = b"60";
pub const TAG_OPEN_CLOSE: &[u8] = b"77";
pub const TAG_MSG_SEQ_NUM: &[u8] = b"34";
pub const TAG_TEST_REQ_ID: &[u8] = b"112";
pub const TAG_EXEC_TYPE: &[u8] = b"150";
pub const TAG_LEAVES_QTY: &[u8] = b"151";
pub const TAG_SECURITY_TYPE: &[u8] = b"167";
pub const TAG_MATURITY_MONTH_YEAR: &[u8] = b"200";
pub const TAG_PUT_OR_CALL: &[u8] = b"201";
pub const TAG_STRIKE_PRICE: &[u8] = b"202";
pub const TAG_CUSTOMER_OR_FIRM: &[u8] = b"204";
pub const TAG_MATURITY_DAY: &[u8] = b"205";
pub const TAG_MATURITY_DATE: &[u8] = b"541";
pub const TAG_CXL_REJ_RESPONSE_TO: &[u8] = b"434";
pub const TAG_ENCRYPT_METHOD: &[u8] = b"98";
pub const TAG_HEART_BT_INT: &[u8] = b"108";

#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct FIXEvent {
    pub comp_id: Arc<str>,
    #[pyo3(get, set)]
    pub payload: FIXPayload,
}

#[gen_stub_pymethods]
#[pymethods]
impl FIXEvent {
    #[getter]
    pub fn comp_id(&self) -> &str {
        &self.comp_id
    }
}

#[gen_stub_pyclass_complex_enum]
#[pyclass()]
#[derive(Debug, Clone)]
pub enum FIXPayload {
    Engine(EngineMessage),
    Business(BusinessMessage),
    Report(ReportMessage),
}

impl FIXPayload {
    pub fn message_type(&self) -> &'static [u8] {
        match self {
            FIXPayload::Engine(msg) => msg.message_type(),
            FIXPayload::Business(msg) => msg.message_type(),
            FIXPayload::Report(msg) => msg.message_type(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            FIXPayload::Engine(msg) => msg.as_bytes(),
            FIXPayload::Business(msg) => msg.as_bytes(),
            FIXPayload::Report(msg) => msg.as_bytes(),
        }
    }
}

#[gen_stub_pyclass_complex_enum]
#[pyclass()]
#[derive(Debug, Clone)]
pub enum ReportMessage {
    ExecutionReport(ExecutionReport),
    OrderCancelReject(OrderCancelReject),
}

impl ReportMessage {
    pub fn message_type(&self) -> &'static [u8] {
        match self {
            ReportMessage::ExecutionReport(_) => ExecutionReport::MESSAGE_TYPE,
            ReportMessage::OrderCancelReject(_) => OrderCancelReject::MESSAGE_TYPE,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            ReportMessage::ExecutionReport(msg) => msg.as_bytes(),
            ReportMessage::OrderCancelReject(msg) => msg.as_bytes(),
        }
    }
}

#[gen_stub_pyclass_complex_enum]
#[pyclass()]
#[derive(Debug, Clone)]
pub enum BusinessMessage {
    NewOrderSingle(NewOrderSingle),
    OrderCancel(OrderCancel),
}

impl BusinessMessage {
    pub fn message_type(&self) -> &'static [u8] {
        match self {
            BusinessMessage::NewOrderSingle(_) => NewOrderSingle::MESSAGE_TYPE,
            BusinessMessage::OrderCancel(_) => OrderCancel::MESSAGE_TYPE,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            BusinessMessage::NewOrderSingle(msg) => msg.as_bytes(),
            BusinessMessage::OrderCancel(msg) => msg.as_bytes(),
        }
    }
}

#[gen_stub_pyclass_complex_enum]
#[pyclass()]
#[derive(Debug, Clone)]
pub enum EngineMessage {
    Logon(Logon),
    Heartbeat(Heartbeat),
    TestRequest(TestRequest),
    ResendRequest(ResendRequest),
}

impl EngineMessage {
    pub fn message_type(&self) -> &'static [u8] {
        match self {
            EngineMessage::Logon(_) => Logon::MESSAGE_TYPE,
            EngineMessage::Heartbeat(_) => Heartbeat::MESSAGE_TYPE,
            EngineMessage::TestRequest(_) => TestRequest::MESSAGE_TYPE,
            EngineMessage::ResendRequest(_) => ResendRequest::MESSAGE_TYPE,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            EngineMessage::Logon(msg) => msg.as_bytes(),
            EngineMessage::Heartbeat(msg) => msg.as_bytes(),
            EngineMessage::TestRequest(msg) => msg.as_bytes(),
            EngineMessage::ResendRequest(msg) => msg.as_bytes(),
        }
    }
}
