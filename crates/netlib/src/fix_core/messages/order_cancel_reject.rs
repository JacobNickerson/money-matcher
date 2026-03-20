use std::str::from_utf8;

use crate::fix_core::{
    iterator::FixIterator,
    messages::{
        FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT, FIXMessage, TAG_CL_ORD_ID, TAG_CXL_REJ_RESPONSE_TO,
        TAG_ORD_STATUS, TAG_ORIG_CL_ORD_ID, TAG_TEXT,
        types::{CxlRejResponseTo, OrdStatus},
    },
};

/// An Order Cancel Reject message is returned by the exchange in the event of an invalid cancel or modify request.
///
/// `MsgType = 9`
#[derive(Debug, Clone)]
pub struct OrderCancelReject {
    pub cl_ord_id: u64,
    /// Status of order that was to have been canceled or modified.
    pub ord_status: OrdStatus,
    /// ClOrdID of the order that was to have been canceled or modified.
    pub orig_cl_ord_id: u64,
    /// Reject reason
    pub text: String,
    pub cxl_rej_response_to: CxlRejResponseTo,
}

impl FIXMessage for OrderCancelReject {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(TAG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORD_STATUS);
        buf.push(b'=');
        buf.push(self.ord_status as u8);
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORIG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_TEXT);
        buf.push(b'=');
        buf.extend_from_slice(self.text.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_CXL_REJ_RESPONSE_TO);
        buf.push(b'=');
        buf.push(self.cxl_rej_response_to as u8);
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut cl_ord_id = None;
        let mut ord_status = None;
        let mut orig_cl_ord_id = None;
        let mut text = None;
        let mut cxl_rej_response_to = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORD_STATUS => {
                    ord_status = value
                        .first()
                        .copied()
                        .and_then(|b| OrdStatus::try_from(b).ok());
                }
                TAG_ORIG_CL_ORD_ID => {
                    orig_cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_TEXT => {
                    text = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_CXL_REJ_RESPONSE_TO => {
                    cxl_rej_response_to = value
                        .first()
                        .copied()
                        .and_then(|b| CxlRejResponseTo::try_from(b).ok());
                }
                _ => {}
            }
        }

        Ok(OrderCancelReject {
            cl_ord_id: cl_ord_id.ok_or("Missing ClOrdID")?,
            ord_status: ord_status.ok_or("Missing OrdStatus")?,
            orig_cl_ord_id: orig_cl_ord_id.ok_or("Missing OrigClOrdID")?,
            text: text.ok_or("Missing Text")?,
            cxl_rej_response_to: cxl_rej_response_to.ok_or("Missing CxlRejResponseTo")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_reject_initial_state() {
        let o = OrderCancelReject {
            cl_ord_id: 1,
            ord_status: OrdStatus::Canceled,
            orig_cl_ord_id: 456,
            text: "reason".to_string(),
            cxl_rej_response_to: CxlRejResponseTo::OrderCancelRequest,
        };

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.ord_status, OrdStatus::Canceled);
        assert_eq!(o.orig_cl_ord_id, 456);
        assert_eq!(o.text, "reason");
        assert_eq!(o.cxl_rej_response_to, CxlRejResponseTo::OrderCancelRequest);
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = OrderCancelReject {
            cl_ord_id: 1,
            ord_status: OrdStatus::Canceled,
            orig_cl_ord_id: 456,
            text: "reason".to_string(),
            cxl_rej_response_to: CxlRejResponseTo::OrderCancelRequest,
        };

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("39=4"));
        assert!(s.contains("41=456"));
        assert!(s.contains("58=reason"));
        assert!(s.contains("434=1"));
    }
}
