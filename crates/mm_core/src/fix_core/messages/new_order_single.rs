use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    iterator::FixIterator,
    messages::{
        FIXMessage, TAG_CL_ORD_ID, TAG_CUSTOMER_OR_FIRM, TAG_HANDL_INST, TAG_MATURITY_DAY,
        TAG_MATURITY_MONTH_YEAR, TAG_OPEN_CLOSE, TAG_ORD_TYPE, TAG_ORDER_QTY, TAG_PRICE,
        TAG_PUT_OR_CALL, TAG_SECURITY_TYPE, TAG_SIDE, TAG_STRIKE_PRICE, TAG_SYMBOL,
        TAG_TRANSACT_TIME,
        types::{CustomerOrFirm, OpenClose, OrdType, PutOrCall, Side},
    },
};
use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use std::str::from_utf8;

/// New Order Single is used to send a regular or Block order.
///
/// `MsgType = D`
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct NewOrderSingle {
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
    pub transact_time: Option<String>,
    pub open_close: OpenClose,
    /// `OPT`
    pub security_type: String,
    pub put_or_call: PutOrCall,
    pub strike_price: u32,
    pub customer_or_firm: CustomerOrFirm,
    pub maturity_day: u8,
}

impl FIXMessage for NewOrderSingle {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(itoa_buf.format(TAG_CL_ORD_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_HANDL_INST).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.handl_inst).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORDER_QTY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORD_TYPE).as_bytes());
        buf.push(b'=');
        buf.push(self.ord_type as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_PRICE).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.price).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_SIDE).as_bytes());
        buf.push(b'=');
        buf.push(self.side as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_SYMBOL).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.symbol.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_TRANSACT_TIME).as_bytes());
        buf.push(b'=');
        if let Some(timestamp) = &self.transact_time {
            buf.extend_from_slice(timestamp.as_bytes());
        } else {
            buf.extend_from_slice(get_timestamp().as_bytes());
        }
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_OPEN_CLOSE).as_bytes());
        buf.push(b'=');
        buf.push(self.open_close as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_SECURITY_TYPE).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.security_type.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_MATURITY_MONTH_YEAR).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_PUT_OR_CALL).as_bytes());
        buf.push(b'=');
        buf.push(self.put_or_call as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_STRIKE_PRICE).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.strike_price).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_CUSTOMER_OR_FIRM).as_bytes());
        buf.push(b'=');
        buf.push(self.customer_or_firm as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_MATURITY_DAY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.maturity_day).as_bytes());
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut cl_ord_id: Option<u64> = None;
        let mut handl_inst: Option<u8> = None;
        let mut qty: Option<u32> = None;
        let mut ord_type: Option<OrdType> = None;
        let mut price: Option<u32> = None;
        let mut side: Option<Side> = None;
        let mut symbol: Option<String> = None;
        let mut transact_time: Option<String> = None;
        let mut open_close: Option<OpenClose> = None;
        let mut security_type: Option<String> = None;
        let mut put_or_call = None;
        let mut strike_price = None;
        let mut customer_or_firm = None;
        let mut maturity_day = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok()),
                TAG_HANDL_INST => handl_inst = from_utf8(value).ok().and_then(|v| v.parse().ok()),
                TAG_ORDER_QTY => qty = from_utf8(value).ok().and_then(|v| v.parse().ok()),
                TAG_ORD_TYPE => ord_type = value.first().and_then(|&b| OrdType::try_from(b).ok()),
                TAG_PRICE => price = from_utf8(value).ok().and_then(|v| v.parse().ok()),
                TAG_SIDE => side = value.first().and_then(|&b| Side::try_from(b).ok()),
                TAG_SYMBOL => symbol = from_utf8(value).ok().map(str::to_owned),
                TAG_TRANSACT_TIME => transact_time = from_utf8(value).ok().map(str::to_owned),
                TAG_OPEN_CLOSE => {
                    open_close = value.first().and_then(|&b| OpenClose::try_from(b).ok())
                }
                TAG_SECURITY_TYPE => security_type = from_utf8(value).ok().map(str::to_owned),
                TAG_PUT_OR_CALL => {
                    put_or_call = value.first().and_then(|&b| PutOrCall::try_from(b).ok())
                }
                TAG_STRIKE_PRICE => {
                    strike_price = from_utf8(value).ok().and_then(|v| v.parse().ok())
                }
                TAG_CUSTOMER_OR_FIRM => {
                    customer_or_firm = value
                        .first()
                        .and_then(|&b| CustomerOrFirm::try_from(b).ok())
                }
                TAG_MATURITY_DAY => {
                    maturity_day = from_utf8(value).ok().and_then(|v| v.parse().ok())
                }
                _ => {}
            }
        }

        Ok(NewOrderSingle {
            cl_ord_id: cl_ord_id.ok_or("Missing ClOrdID")?,
            handl_inst: handl_inst.ok_or("Missing HandlInst")?,
            qty: qty.ok_or("Missing OrderQty")?,
            ord_type: ord_type.ok_or("Missing OrdType")?,
            price: price.ok_or("Missing Price")?,
            side: side.ok_or("Missing Side")?,
            symbol: symbol.ok_or("Missing Symbol")?,
            transact_time: Some(transact_time.ok_or("Missing TransactTime")?),
            open_close: open_close.ok_or("Missing OpenClose")?,
            security_type: security_type.ok_or("Missing SecurityType")?,
            put_or_call: put_or_call.ok_or("Missing PutOrCall")?,
            strike_price: strike_price.ok_or("Missing StrikePrice")?,
            customer_or_firm: customer_or_firm.ok_or("Missing CustomerOrFirm")?,
            maturity_day: maturity_day.ok_or("Missing MaturityDay")?,
        })
    }
}
