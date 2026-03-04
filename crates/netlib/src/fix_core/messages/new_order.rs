use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    messages::{
        FIX_MESSAGE_TYPE_NEW_ORDER, FixMessage, TAG_CL_ORD_ID, TAG_CUSTOMER_OR_FIRM,
        TAG_HANDL_INST, TAG_MATURITY_DAY, TAG_MATURITY_MONTH_YEAR, TAG_OPEN_CLOSE, TAG_ORD_TYPE,
        TAG_ORDER_QTY, TAG_PRICE, TAG_PUT_OR_CALL, TAG_SECURITY_TYPE, TAG_SIDE, TAG_STRIKE_PRICE,
        TAG_SYMBOL, TAG_TRANSACT_TIME,
        types::{OpenClose, OrdType, Side},
    },
};

/// New Order Single is used to send a regular or Block order.
///
/// `MsgType = D`
pub struct NewOrder {
    /// Maximum 20 characters. Any value exceeding 20 characters will be rejected.
    pub cl_ord_id: u64,
    /// Required by FIX protocol, but ignored by ISE.
    pub handl_inst: u8,
    pub qty: u32,
    pub ord_type: OrdType,
    /// Required if OrdType = 2 or 4.
    pub price: u32,
    pub side: Side,
    /// OSI symbol for a series.
    pub symbol: String,
    pub open_close: OpenClose,
    /// `OPT`
    pub security_type: String,
}

impl NewOrder {
    pub fn new(
        cl_ord_id: u64,
        handl_inst: u8,
        qty: u32,
        ord_type: OrdType,
        price: u32,
        side: Side,
        symbol: String,
        open_close: OpenClose,
        security_type: String,
    ) -> Self {
        Self {
            cl_ord_id,
            handl_inst,
            qty,
            ord_type,
            price,
            side,
            symbol,
            open_close,
            security_type,
        }
    }
}

impl FixMessage for NewOrder {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_NEW_ORDER;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        // 11 - ClOrdID
        buf.extend_from_slice(TAG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        // 21 - HandlInst
        buf.extend_from_slice(TAG_HANDL_INST);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.handl_inst).as_bytes());
        buf.push(0x01);

        // 38 - OrderQty
        buf.extend_from_slice(TAG_ORDER_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        // 40 - OrdType
        buf.extend_from_slice(TAG_ORD_TYPE);
        buf.push(b'=');
        buf.push(self.ord_type as u8);
        buf.push(0x01);

        // 44 - Price
        buf.extend_from_slice(TAG_PRICE);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.price).as_bytes());
        buf.push(0x01);

        // 54 - Side
        buf.extend_from_slice(TAG_SIDE);
        buf.push(b'=');
        buf.push(self.side as u8);
        buf.push(0x01);

        // 55 - Symbol
        buf.extend_from_slice(TAG_SYMBOL);
        buf.push(b'=');
        buf.extend_from_slice(self.symbol.as_bytes());
        buf.push(0x01);

        // 60 - TransactTime
        buf.extend_from_slice(TAG_TRANSACT_TIME);
        buf.push(b'=');
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        // 77 - OpenClose
        buf.extend_from_slice(TAG_OPEN_CLOSE);
        buf.push(b'=');
        buf.push(self.open_close as u8);
        buf.push(0x01);

        // 167 - SecurityType
        buf.extend_from_slice(TAG_SECURITY_TYPE);
        buf.push(b'=');
        buf.extend_from_slice(self.security_type.as_bytes());
        buf.push(0x01);

        // 200 - MaturityMonthYear
        buf.extend_from_slice(TAG_MATURITY_MONTH_YEAR);
        buf.push(b'=');
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        // 201 - PutOrCall
        buf.extend_from_slice(TAG_PUT_OR_CALL);
        buf.push(b'=');
        buf.push(b'1');
        buf.push(0x01);

        // 202 - StrikePrice
        buf.extend_from_slice(TAG_STRIKE_PRICE);
        buf.push(b'=');
        buf.extend_from_slice(b"10");
        buf.push(0x01);

        // 204 - CustomerOrFirm
        buf.extend_from_slice(TAG_CUSTOMER_OR_FIRM);
        buf.push(b'=');
        buf.push(b'0');
        buf.push(0x01);

        // 205 - MaturityDay
        buf.extend_from_slice(TAG_MATURITY_DAY);
        buf.push(b'=');
        buf.extend_from_slice(b"10");
        buf.push(0x01);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_order_initial_state() {
        let o = NewOrder::new(
            1,
            1,
            123,
            OrdType::Limit,
            12345,
            Side::Buy,
            "str1".to_string(),
            OpenClose::Open,
            "OPT".to_string(),
        );

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.handl_inst, 1);
        assert_eq!(o.qty, 123);
        assert_eq!(o.ord_type, OrdType::Limit);
        assert_eq!(o.price, 12345);
        assert_eq!(o.side, Side::Buy);
        assert_eq!(o.symbol, "str1");
        assert_eq!(o.open_close, OpenClose::Open);
        assert_eq!(o.security_type, "OPT");
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = NewOrder::new(
            1,
            1,
            123,
            OrdType::Limit,
            12345,
            Side::Buy,
            "str1".to_string(),
            OpenClose::Open,
            "OPT".to_string(),
        );

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("21=1"));
        assert!(s.contains("38=123"));
        assert!(s.contains("40=2"));
        assert!(s.contains("44=12345"));
        assert!(s.contains("54=1"));
        assert!(s.contains("55=str1"));
        assert!(s.contains("201=1"));
        assert!(s.contains("202=10"));
        assert!(s.contains("204=0"));
        assert!(s.contains("205=10"));
    }
}
