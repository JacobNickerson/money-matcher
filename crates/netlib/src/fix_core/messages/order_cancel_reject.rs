use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    messages::{
        FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT, FixMessage,
        types::{CxlRejResponseTo, OrdStatus},
    },
};

/// An Order Cancel Reject message is returned by the exchange in the event of an invalid cancel
/// or modify request.
///
/// `MsgType = 9`
pub struct OrderCancelReject {
    pub cl_ord_id: u64,
    /// Status of order that was to have been canceled or modified.
    pub ord_status: OrdStatus,
    /// ClOrdID of the order that was to have been canceled or modified.
    pub orig_cl_ord_id: u64,
    /// Reject reason
    pub text: String,
    /// `1` = Order Cancel Request, `2` = Order Cancel Replace Request
    pub cxl_rej_response_to: CxlRejResponseTo,
}

impl OrderCancelReject {
    pub fn new(
        cl_ord_id: u64,
        ord_status: OrdStatus,
        orig_cl_ord_id: u64,
        text: String,
        cxl_rej_response_to: CxlRejResponseTo,
    ) -> Self {
        Self {
            cl_ord_id,
            ord_status,
            orig_cl_ord_id,
            text,
            cxl_rej_response_to,
        }
    }
}

impl FixMessage for OrderCancelReject {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        // 11 - ClOrdID
        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        // 39 - OrdStatus
        buf.extend_from_slice(b"39=");
        buf.extend_from_slice(itoa_buf.format(self.ord_status as u8).as_bytes());
        buf.push(0x01);

        // 41 - OrigClOrdID
        buf.extend_from_slice(b"41=");
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        // 58 - Text: Reject reason
        buf.extend_from_slice(b"58=");
        buf.extend_from_slice(self.text.as_bytes());
        buf.push(0x01);

        // 434 - CxlRejResponseTo: 1=Order Cancel Request, 2=Order Cancel Replace Request
        buf.extend_from_slice(b"434=");
        buf.extend_from_slice(itoa_buf.format(self.cxl_rej_response_to as u8).as_bytes());
        buf.push(0x01);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_reject_initial_state() {
        let o = OrderCancelReject::new(
            1,
            OrdStatus::Canceled,
            456,
            "reason".to_string(),
            CxlRejResponseTo::OrderCancelRequest,
        );

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.ord_status, OrdStatus::Canceled);
        assert_eq!(o.orig_cl_ord_id, 456);
        assert_eq!(o.text, "reason");
        assert_eq!(o.cxl_rej_response_to, CxlRejResponseTo::OrderCancelRequest);
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = OrderCancelReject::new(
            1,
            OrdStatus::Canceled,
            456,
            "reason".to_string(),
            CxlRejResponseTo::OrderCancelRequest,
        );

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("39=4"));
        assert!(s.contains("41=456"));
        assert!(s.contains("58=reason"));
        assert!(s.contains("434=1"));
    }
}
