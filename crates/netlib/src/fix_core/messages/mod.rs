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

pub const TAG_BEGIN_STRING: &'static [u8] = b"8";
pub const TAG_BODY_LENGTH: &'static [u8] = b"9";
pub const TAG_CHECKSUM: &'static [u8] = b"10";
pub const TAG_CL_ORD_ID: &'static [u8] = b"11";
pub const TAG_CUM_QTY: &'static [u8] = b"14";
pub const TAG_EXEC_ID: &'static [u8] = b"17";
pub const TAG_EXEC_TRANS_TYPE: &'static [u8] = b"20";
pub const TAG_HANDL_INST: &'static [u8] = b"21";
pub const TAG_MSG_TYPE: &'static [u8] = b"35";
pub const TAG_ORDER_ID: &'static [u8] = b"37";
pub const TAG_ORDER_QTY: &'static [u8] = b"38";
pub const TAG_ORD_STATUS: &'static [u8] = b"39";
pub const TAG_ORD_TYPE: &'static [u8] = b"40";
pub const TAG_ORIG_CL_ORD_ID: &'static [u8] = b"41";
pub const TAG_PRICE: &'static [u8] = b"44";
pub const TAG_SECURITY_ID: &'static [u8] = b"48";
pub const TAG_SENDER_COMP_ID: &'static [u8] = b"49";
pub const TAG_SENDING_TIME: &'static [u8] = b"52";
pub const TAG_SIDE: &'static [u8] = b"54";
pub const TAG_SYMBOL: &'static [u8] = b"55";
pub const TAG_TARGET_COMP_ID: &'static [u8] = b"56";
pub const TAG_TEXT: &'static [u8] = b"58";
pub const TAG_TRANSACT_TIME: &'static [u8] = b"60";
pub const TAG_OPEN_CLOSE: &'static [u8] = b"77";
pub const TAG_MSG_SEQ_NUM: &'static [u8] = b"34";
pub const TAG_EXEC_TYPE: &'static [u8] = b"150";
pub const TAG_LEAVES_QTY: &'static [u8] = b"151";
pub const TAG_SECURITY_TYPE: &'static [u8] = b"167";
pub const TAG_MATURITY_MONTH_YEAR: &'static [u8] = b"200";
pub const TAG_PUT_OR_CALL: &'static [u8] = b"201";
pub const TAG_STRIKE_PRICE: &'static [u8] = b"202";
pub const TAG_CUSTOMER_OR_FIRM: &'static [u8] = b"204";
pub const TAG_MATURITY_DAY: &'static [u8] = b"205";
pub const TAG_MATURITY_DATE: &'static [u8] = b"541";
pub const TAG_CXL_REJ_RESPONSE_TO: &'static [u8] = b"434";
