use crate::fix_core::{
    helpers::get_timestamp,
    messages::{
        FIX_MESSAGE_TYPE_ORDER_CANCEL, FixMessage, TAG_CL_ORD_ID, TAG_ORDER_QTY,
        TAG_ORIG_CL_ORD_ID, TAG_TRANSACT_TIME,
    },
};
/// The Order Cancel Request message is used to cancel a regular or multi-leg order.
///
/// `MsgType = F`
pub struct OrderCancel {
    /// Maximum 20 characters. Any value exceeding 20 characters will be rejected.
    pub cl_ord_id: u64,
    /// Number of known open contracts.
    pub qty: u32,
    /// ClOrdID of the order to be canceled.
    pub orig_cl_ord_id: u64,
}

impl OrderCancel {
    pub fn new(cl_ord_id: u64, qty: u32, orig_cl_ord_id: u64) -> Self {
        Self {
            cl_ord_id,
            qty,
            orig_cl_ord_id,
        }
    }
}

impl FixMessage for OrderCancel {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_ORDER_CANCEL;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(TAG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORDER_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORIG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_TRANSACT_TIME);
        buf.push(b'=');
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_initial_state() {
        let o = OrderCancel::new(1, 123, 456);

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.qty, 123);
        assert_eq!(o.orig_cl_ord_id, 456);
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = OrderCancel::new(1, 123, 456);

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("38=123"));
        assert!(s.contains("41=456"));
    }
}
