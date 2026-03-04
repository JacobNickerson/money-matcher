use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    messages::{
        FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE, FixMessage,
        types::{CustomerOrFirm, OpenClose, OrdType, PutOrCall, Side},
    },
};

/// The Order Cancel Replace Request message is used to modify a regular order.
///
/// `MsgType = G`
pub struct OrderCancelReplace {
    /// Maximum 20 characters. Any value exceeding 20 characters will be rejected.
    pub cl_ord_id: u64,
    /// Ignored by ISE.
    pub handl_inst: u8,
    pub qty: u32,
    pub ord_type: OrdType,
    /// ClOrdID of the order to be modified.
    pub orig_cl_ord_id: u64,
    /// Must match the original order.
    pub side: Side,
    /// Must match the original order.
    pub symbol: String,
    /// Must match the original order.
    pub open_close: OpenClose,
    /// Must match the original order.
    pub security_type: String,
    /// Must match the original order.
    pub put_or_call: PutOrCall,
    /// Must match the original order.
    pub strike_price: u32,
    /// Must match the original order.
    pub customer_or_firm: CustomerOrFirm,
}

impl OrderCancelReplace {
    pub fn new(
        cl_ord_id: u64,
        handl_inst: u8,
        qty: u32,
        ord_type: OrdType,
        orig_cl_ord_id: u64,
        side: Side,
        symbol: String,
        open_close: OpenClose,
        security_type: String,
        put_or_call: PutOrCall,
        strike_price: u32,
        customer_or_firm: CustomerOrFirm,
    ) -> Self {
        Self {
            cl_ord_id,
            handl_inst,
            qty,
            ord_type,
            orig_cl_ord_id,
            side,
            symbol,
            open_close,
            security_type,
            put_or_call,
            strike_price,
            customer_or_firm,
        }
    }
}

impl FixMessage for OrderCancelReplace {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        // 11 - ClOrdID
        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        // 21 - HandlInst: Ignored by ISE.
        buf.extend_from_slice(b"21=");
        buf.extend_from_slice(itoa_buf.format(self.handl_inst).as_bytes());
        buf.push(0x01);

        // 38 - OrderQty
        buf.extend_from_slice(b"38=");
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        // 40 - OrdType
        buf.extend_from_slice(b"40=");
        buf.push(self.ord_type as u8);
        buf.push(0x01);

        // 41 - OrigClOrdID
        buf.extend_from_slice(b"41=");
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        // 54 - Side: Must match the original order.
        buf.extend_from_slice(b"54=");
        buf.push(self.side as u8);
        buf.push(0x01);

        // 55 - Symbol: Must match the original order.
        buf.extend_from_slice(b"55=");
        buf.extend_from_slice(self.symbol.as_bytes());
        buf.push(0x01);

        // 60 - TransactTime: YYYYMMDD-HH:MM:SS.sss (milliseconds)
        buf.extend_from_slice(b"60=");
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        // 77 - OpenClose: Must match the original order.
        buf.extend_from_slice(b"77=");
        buf.push(self.open_close as u8);
        buf.push(0x01);

        // 167 - SecurityType: Must match the original order.
        buf.extend_from_slice(b"167=");
        buf.extend_from_slice(self.security_type.as_bytes());
        buf.push(0x01);

        // 200 - MaturityMonthYear: Must match the original order.
        // If this tag and Maturity Date is specified the year and month must match,
        // otherwise the order will be rejected.
        // Required unless Maturity Date (tag 541) is specified.
        buf.extend_from_slice(b"200=");
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        // 201 - PutOrCall: Must match the original order.
        buf.extend_from_slice(b"201=");
        buf.push(self.put_or_call as u8);
        buf.push(0x01);

        // 202 - StrikePrice: Must match the original order.
        buf.extend_from_slice(b"202=");
        buf.extend_from_slice(itoa_buf.format(self.strike_price).as_bytes());
        buf.push(0x01);

        // 204 - CustomerOrFirm: Must match the original order.
        buf.extend_from_slice(b"204=");
        buf.push(self.customer_or_firm as u8);
        buf.push(0x01);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_replace_initial_state() {
        let o = OrderCancelReplace::new(
            1,
            1,
            123,
            OrdType::Limit,
            456,
            Side::Buy,
            "str1".to_string(),
            OpenClose::Open,
            "OPT".to_string(),
            PutOrCall::Call,
            10,
            CustomerOrFirm::Customer,
        );

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.handl_inst, 1);
        assert_eq!(o.qty, 123);
        assert_eq!(o.ord_type, OrdType::Limit);
        assert_eq!(o.orig_cl_ord_id, 456);
        assert_eq!(o.side, Side::Buy);
        assert_eq!(o.symbol, "str1");
        assert_eq!(o.open_close, OpenClose::Open);
        assert_eq!(o.security_type, "OPT");
        assert_eq!(o.put_or_call, PutOrCall::Call);
        assert_eq!(o.strike_price, 10);
        assert_eq!(o.customer_or_firm, CustomerOrFirm::Customer);
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = OrderCancelReplace::new(
            1,
            1,
            123,
            OrdType::Limit,
            456,
            Side::Buy,
            "str1".to_string(),
            OpenClose::Open,
            "OPT".to_string(),
            PutOrCall::Call,
            10,
            CustomerOrFirm::Customer,
        );

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("21=1"));
        assert!(s.contains("38=123"));
        assert!(s.contains("40=2"));
        assert!(s.contains("41=456"));
        assert!(s.contains("54=1"));
        assert!(s.contains("55=str1"));
        assert!(s.contains("77="));
        assert!(s.contains("167=OPT"));
        assert!(s.contains("201=1"));
        assert!(s.contains("202=10"));
        assert!(s.contains("204=0"));
    }
}
