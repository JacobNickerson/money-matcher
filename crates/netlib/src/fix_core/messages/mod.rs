pub trait FixMessage {
    const MESSAGE_TYPE: &'static [u8];
    fn as_bytes(&self) -> Vec<u8>;
}

pub struct FixFrame {
    pub msg_type: &'static [u8],
    pub body: Vec<u8>,
}

pub mod execution_report;
pub mod new_order;
pub mod order_cancel;
pub mod order_cancel_reject;
pub mod order_cancel_replace;
pub mod types;

pub const FIX_MESSAGE_TYPE_EXECUTION_REPORT: &'static [u8] = b"8";
pub const FIX_MESSAGE_TYPE_NEW_ORDER: &'static [u8] = b"D";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT: &'static [u8] = b"9";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE: &'static [u8] = b"G";
pub const FIX_MESSAGE_TYPE_ORDER_CANCEL: &'static [u8] = b"F";
