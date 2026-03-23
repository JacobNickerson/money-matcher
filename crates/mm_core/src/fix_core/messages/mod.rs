use crate::{
    fix_core::messages::{
        execution_report::ExecutionReport, heartbeat::Heartbeat, logon::Logon,
        new_order_single::NewOrderSingle, order_cancel::OrderCancel,
        order_cancel_reject::OrderCancelReject, order_cancel_replace::OrderCancelReplace,
        resend_request::ResendRequest, test_request::TestRequest,
    },
    lob_core::market_orders::{Order, OrderType},
};
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pymethods};
use std::sync::Arc;

pub trait FIXMessage {
    fn as_bytes(&self) -> Vec<u8>;
    fn from_bytes(msg: &[u8]) -> Result<Self, &str>
    where
        Self: Sized;
}

pub trait FIXBusinessMessage {
    fn to_order(self) -> Order;
    fn from_order(order: &Order) -> Result<Self, &'static str>
    where
        Self: Sized;
}

pub struct FixFrame {
    pub msg_type: u8,
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

pub const FIX_MESSAGE_TYPE_HEARTBEAT: u8 = b'0';
pub const FIX_MESSAGE_TYPE_TEST_REQUEST: u8 = b'1';
pub const FIX_MESSAGE_TYPE_EXECUTION_REPORT: u8 = b'8';
pub const FIX_MESSAGE_TYPE_NEW_ORDER: u8 = b'D';
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT: u8 = b'9';
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE: u8 = b'G';
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL: u8 = b'F';
pub const FIX_MESSAGE_TYPE_LOGON: u8 = b'A';
pub const FIX_MESSAGE_TYPE_RESEND_REQUEST: u8 = b'2';

pub const TAG_BEGIN_SEQ_NO: u16 = 7;
pub const TAG_BEGIN_STRING: u16 = 8;
pub const TAG_BODY_LENGTH: u16 = 9;
pub const TAG_CHECKSUM: u16 = 10;
pub const TAG_CL_ORD_ID: u16 = 11;
pub const TAG_CUM_QTY: u16 = 14;
pub const TAG_END_SEQ_NO: u16 = 16;
pub const TAG_EXEC_ID: u16 = 17;
pub const TAG_EXEC_TRANS_TYPE: u16 = 20;
pub const TAG_HANDL_INST: u16 = 21;
pub const TAG_MSG_SEQ_NUM: u16 = 34;
pub const TAG_MSG_TYPE: u16 = 35;
pub const TAG_ORDER_ID: u16 = 37;
pub const TAG_ORDER_QTY: u16 = 38;
pub const TAG_ORD_STATUS: u16 = 39;
pub const TAG_ORD_TYPE: u16 = 40;
pub const TAG_ORIG_CL_ORD_ID: u16 = 41;
pub const TAG_POSS_DUP_FLAG: u16 = 43;
pub const TAG_PRICE: u16 = 44;
pub const TAG_SECURITY_ID: u16 = 48;
pub const TAG_SENDER_COMP_ID: u16 = 49;
pub const TAG_SENDING_TIME: u16 = 52;
pub const TAG_SIDE: u16 = 54;
pub const TAG_SYMBOL: u16 = 55;
pub const TAG_TARGET_COMP_ID: u16 = 56;
pub const TAG_TEXT: u16 = 58;
pub const TAG_TRANSACT_TIME: u16 = 60;
pub const TAG_OPEN_CLOSE: u16 = 77;
pub const TAG_ENCRYPT_METHOD: u16 = 98;
pub const TAG_HEART_BT_INT: u16 = 108;
pub const TAG_TEST_REQ_ID: u16 = 112;
pub const TAG_EXEC_TYPE: u16 = 150;
pub const TAG_LEAVES_QTY: u16 = 151;
pub const TAG_SECURITY_TYPE: u16 = 167;
pub const TAG_MATURITY_MONTH_YEAR: u16 = 200;
pub const TAG_PUT_OR_CALL: u16 = 201;
pub const TAG_STRIKE_PRICE: u16 = 202;
pub const TAG_CUSTOMER_OR_FIRM: u16 = 204;
pub const TAG_MATURITY_DAY: u16 = 205;
pub const TAG_CXL_REJ_RESPONSE_TO: u16 = 434;
pub const TAG_MATURITY_DATE: u16 = 541;

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
    pub fn message_type(&self) -> u8 {
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
    pub fn message_type(&self) -> u8 {
        match self {
            ReportMessage::ExecutionReport(_) => FIX_MESSAGE_TYPE_EXECUTION_REPORT,
            ReportMessage::OrderCancelReject(_) => FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT,
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
    OrderCancelReplace(OrderCancelReplace),
}

impl BusinessMessage {
    pub fn message_type(&self) -> u8 {
        match self {
            BusinessMessage::NewOrderSingle(_) => FIX_MESSAGE_TYPE_NEW_ORDER,
            BusinessMessage::OrderCancel(_) => FIX_MESSAGE_TYPE_ORDER_CANCEL,
            BusinessMessage::OrderCancelReplace(_) => FIX_MESSAGE_TYPE_ORDER_CANCEL,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            BusinessMessage::NewOrderSingle(msg) => msg.as_bytes(),
            BusinessMessage::OrderCancel(msg) => msg.as_bytes(),
            BusinessMessage::OrderCancelReplace(msg) => msg.as_bytes(),
        }
    }
}

impl FIXBusinessMessage for BusinessMessage {
    fn to_order(self) -> Order {
        match self {
            BusinessMessage::NewOrderSingle(msg) => msg.to_order(),
            BusinessMessage::OrderCancel(msg) => msg.to_order(),
            BusinessMessage::OrderCancelReplace(msg) => msg.to_order(),
        }
    }

    fn from_order(order: &Order) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        match order.kind {
            OrderType::Limit { .. } => Ok(BusinessMessage::NewOrderSingle(
                NewOrderSingle::from_order(order)?,
            )),
            OrderType::Cancel { .. } => Ok(BusinessMessage::OrderCancel(OrderCancel::from_order(
                order,
            )?)),
            OrderType::Update { .. } => Ok(BusinessMessage::OrderCancelReplace(
                OrderCancelReplace::from_order(order)?,
            )),
            _ => Err("Unsupported order kind"),
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
    pub fn message_type(&self) -> u8 {
        match self {
            EngineMessage::Logon(_) => FIX_MESSAGE_TYPE_LOGON,
            EngineMessage::Heartbeat(_) => FIX_MESSAGE_TYPE_HEARTBEAT,
            EngineMessage::TestRequest(_) => FIX_MESSAGE_TYPE_TEST_REQUEST,
            EngineMessage::ResendRequest(_) => FIX_MESSAGE_TYPE_RESEND_REQUEST,
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
