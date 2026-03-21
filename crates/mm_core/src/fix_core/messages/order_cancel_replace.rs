use std::str::from_utf8;

use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    iterator::FixIterator,
    messages::{
        FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE, FIXMessage, TAG_CL_ORD_ID, TAG_CUSTOMER_OR_FIRM,
        TAG_HANDL_INST, TAG_MATURITY_MONTH_YEAR, TAG_OPEN_CLOSE, TAG_ORD_TYPE, TAG_ORDER_QTY,
        TAG_ORIG_CL_ORD_ID, TAG_PUT_OR_CALL, TAG_SECURITY_TYPE, TAG_SIDE, TAG_STRIKE_PRICE,
        TAG_SYMBOL, TAG_TRANSACT_TIME,
        types::{CustomerOrFirm, OpenClose, OrdType, PutOrCall, Side},
    },
};

/// The Order Cancel Replace Request message is used to modify a regular order.
///
/// `MsgType = G`
#[derive(Debug, Clone)]
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

impl FIXMessage for OrderCancelReplace {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_ORDER_CANCEL_REPLACE;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(TAG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_HANDL_INST);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.handl_inst).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORDER_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORD_TYPE);
        buf.push(b'=');
        buf.push(self.ord_type as u8);
        buf.push(0x01);

        buf.extend_from_slice(TAG_ORIG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_SIDE);
        buf.push(b'=');
        buf.push(self.side as u8);
        buf.push(0x01);

        buf.extend_from_slice(TAG_SYMBOL);
        buf.push(b'=');
        buf.extend_from_slice(self.symbol.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_TRANSACT_TIME);
        buf.push(b'=');
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_OPEN_CLOSE);
        buf.push(b'=');
        buf.push(self.open_close as u8);
        buf.push(0x01);

        buf.extend_from_slice(TAG_SECURITY_TYPE);
        buf.push(b'=');
        buf.extend_from_slice(self.security_type.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_MATURITY_MONTH_YEAR);
        buf.push(b'=');
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_PUT_OR_CALL);
        buf.push(b'=');
        buf.push(self.put_or_call as u8);
        buf.push(0x01);

        buf.extend_from_slice(TAG_STRIKE_PRICE);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.strike_price).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(TAG_CUSTOMER_OR_FIRM);
        buf.push(b'=');
        buf.push(self.customer_or_firm as u8);
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut cl_ord_id = None;
        let mut handl_inst = None;
        let mut qty = None;
        let mut ord_type = None;
        let mut orig_cl_ord_id = None;
        let mut side = None;
        let mut symbol = None;
        let mut open_close = None;
        let mut security_type = None;
        let mut put_or_call = None;
        let mut strike_price = None;
        let mut customer_or_firm = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_HANDL_INST => {
                    handl_inst = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORDER_QTY => {
                    qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORD_TYPE => {
                    ord_type = value
                        .first()
                        .copied()
                        .and_then(|b| OrdType::try_from(b).ok());
                }
                TAG_ORIG_CL_ORD_ID => {
                    orig_cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_SIDE => {
                    side = value.first().copied().and_then(|b| Side::try_from(b).ok());
                }
                TAG_SYMBOL => {
                    symbol = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_OPEN_CLOSE => {
                    open_close = value
                        .first()
                        .copied()
                        .and_then(|b| OpenClose::try_from(b).ok());
                }
                TAG_SECURITY_TYPE => {
                    security_type = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_PUT_OR_CALL => {
                    put_or_call = value
                        .first()
                        .copied()
                        .and_then(|b| PutOrCall::try_from(b).ok());
                }
                TAG_STRIKE_PRICE => {
                    strike_price = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_CUSTOMER_OR_FIRM => {
                    customer_or_firm = value
                        .first()
                        .copied()
                        .and_then(|b| CustomerOrFirm::try_from(b).ok());
                }
                _ => {}
            }
        }

        Ok(OrderCancelReplace {
            cl_ord_id: cl_ord_id.ok_or("Missing ClOrdID")?,
            handl_inst: handl_inst.ok_or("Missing HandlInst")?,
            qty: qty.ok_or("Missing Qty")?,
            ord_type: ord_type.ok_or("Missing OrdType")?,
            orig_cl_ord_id: orig_cl_ord_id.ok_or("Missing OrigClOrdID")?,
            side: side.ok_or("Missing Side")?,
            symbol: symbol.ok_or("Missing Symbol")?,
            open_close: open_close.ok_or("Missing OpenClose")?,
            security_type: security_type.ok_or("Missing SecurityType")?,
            put_or_call: put_or_call.ok_or("Missing PutOrCall")?,
            strike_price: strike_price.ok_or("Missing StrikePrice")?,
            customer_or_firm: customer_or_firm.ok_or("Missing CustomerOrFirm")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_replace_initial_state() {
        let o = OrderCancelReplace {
            cl_ord_id: 1,
            handl_inst: 1,
            qty: 123,
            ord_type: OrdType::Limit,
            orig_cl_ord_id: 456,
            side: Side::Buy,
            symbol: "str1".to_string(),
            open_close: OpenClose::Open,
            security_type: "OPT".to_string(),
            put_or_call: PutOrCall::Call,
            strike_price: 10,
            customer_or_firm: CustomerOrFirm::Customer,
        };

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
        let o = OrderCancelReplace {
            cl_ord_id: 1,
            handl_inst: 1,
            qty: 123,
            ord_type: OrdType::Limit,
            orig_cl_ord_id: 456,
            side: Side::Buy,
            symbol: "str1".to_string(),
            open_close: OpenClose::Open,
            security_type: "OPT".to_string(),
            put_or_call: PutOrCall::Call,
            strike_price: 10,
            customer_or_firm: CustomerOrFirm::Customer,
        };

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
