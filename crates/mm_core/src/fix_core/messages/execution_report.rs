use crate::{
    fix_core::{
        helpers::get_timestamp,
        iterator::FixIterator,
        messages::{
            FIXMessage, TAG_CL_ORD_ID, TAG_CUM_QTY, TAG_CUSTOMER_OR_FIRM, TAG_EXEC_ID,
            TAG_EXEC_TRANS_TYPE, TAG_EXEC_TYPE, TAG_LEAVES_QTY, TAG_MATURITY_DATE, TAG_OPEN_CLOSE,
            TAG_ORD_STATUS, TAG_ORDER_ID, TAG_ORDER_QTY, TAG_PUT_OR_CALL, TAG_SECURITY_ID,
            TAG_SECURITY_TYPE, TAG_SIDE, TAG_STRIKE_PRICE, TAG_SYMBOL, TAG_TRANSACT_TIME,
            types::{
                CustomerOrFirm, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side,
            },
        },
    },
    lob_core::{
        market_events::{ClientEvent, ClientEventType},
        market_orders::OrderSide,
    },
};
use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use std::str::from_utf8;

/// The Execution Report message is used to:
/// • confirm the receipt of an order
/// • confirm changes to an existing order
/// • confirm cancelation of an existing order
/// • relay order status information
/// • relay fill information on working orders
/// • reject orders
/// • report trade busts or other post-trade corrections
///
/// `MsgType = 8`
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub cl_ord_id: u64,
    pub cum_qty: u32,
    pub exec_id: String,
    pub exec_trans_type: ExecTransType,
    pub order_id: String,
    pub order_qty: u32,
    pub ord_status: OrdStatus,
    pub security_id: String,
    pub side: Side,
    pub symbol: String,
    pub open_close: OpenClose,
    pub exec_type: ExecType,
    pub leaves_qty: u32,
    pub security_type: String,
    pub put_or_call: PutOrCall,
    pub strike_price: u32,
    pub customer_or_firm: CustomerOrFirm,
    pub maturity_date: String,
}

impl From<ClientEvent> for ExecutionReport {
    fn from(event: ClientEvent) -> ExecutionReport {
        if event.kind == ClientEventType::Rejected {
            return ExecutionReport {
                cl_ord_id: 0,
                cum_qty: 0,
                exec_id: "".to_string(),
                exec_trans_type: ExecTransType::Status,
                order_id: "".to_string(),
                order_qty: 0,
                ord_status: OrdStatus::Rejected,
                security_id: "".to_string(),
                side: Side::Buy,
                symbol: "".to_string(), // PLACEHOLDER
                open_close: OpenClose::Close,
                exec_type: ExecType::Rejected,
                leaves_qty: 0,
                security_type: "".to_string(),
                put_or_call: PutOrCall::Call,
                strike_price: 0,
                customer_or_firm: CustomerOrFirm::Customer, // PLACEHOLDER
                maturity_date: "".to_string(),
            };
        }
        ExecutionReport {
            cl_ord_id: event.client_id,
            cum_qty: match event.kind {
                ClientEventType::Accepted(qty) => qty,
                ClientEventType::PartiallyFilled(qty) => qty,
                _ => 0,
            },
            exec_id: event.id.to_string(),
            exec_trans_type: match event.kind {
                ClientEventType::Accepted(_) => ExecTransType::New,
                ClientEventType::Rejected => ExecTransType::New,
                ClientEventType::Updated => ExecTransType::New,
                ClientEventType::Canceled => ExecTransType::New,
                ClientEventType::PartiallyFilled(_) => ExecTransType::New,
                ClientEventType::Filled => ExecTransType::New,
            },
            order_id: event.order_id.to_string(),
            order_qty: match event.kind {
                ClientEventType::Accepted(qty) => qty,
                _ => 0,
            },
            ord_status: match event.kind {
                ClientEventType::Accepted(_) => OrdStatus::New,
                ClientEventType::Canceled => OrdStatus::Canceled,
                ClientEventType::Filled => OrdStatus::Filled,
                ClientEventType::PartiallyFilled(_) => OrdStatus::PartiallyFilled,
                ClientEventType::Updated => OrdStatus::Replaced,
                ClientEventType::Rejected => OrdStatus::Rejected,
            },
            security_id: "".to_string(), // PLACEHOLDER, NOTE: We only support one instrument type
            side: match event.order_side {
                OrderSide::Bid => Side::Buy,
                OrderSide::Ask => Side::Sell,
            },
            symbol: "".to_string(),
            open_close: OpenClose::Open, // TODO: Placeholder
            exec_type: match event.kind {
                ClientEventType::Accepted(_) => ExecType::New,
                ClientEventType::Canceled => ExecType::Canceled,
                ClientEventType::Filled => ExecType::Filled,
                ClientEventType::PartiallyFilled(_) => ExecType::PartiallyFilled,
                ClientEventType::Updated => ExecType::Replace,
                ClientEventType::Rejected => ExecType::Rejected,
            },
            leaves_qty: match event.kind {
                ClientEventType::PartiallyFilled(qty) => qty,
                _ => 0,
            },
            security_type: "Stock".to_string(), // PLACEHOLDER, NOTE: We only support one instrument type
            put_or_call: PutOrCall::Call, // PLACEHOLDER, NOTE: Only relevant for options, which are not currently supported
            strike_price: 0, // PLACEHOLDER, NOTE: Only relevant for options, which are not currently supported
            customer_or_firm: CustomerOrFirm::Customer,
            maturity_date: "01/01/9999".to_string(), // PLACEHOLDER
        }
    }
}

impl FIXMessage for ExecutionReport {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(itoa_buf.format(TAG_CL_ORD_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_CUM_QTY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cum_qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_EXEC_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.exec_id.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_EXEC_TRANS_TYPE).as_bytes());
        buf.push(b'=');
        buf.push(self.exec_trans_type as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORDER_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.order_id.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORDER_QTY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.order_qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORD_STATUS).as_bytes());
        buf.push(b'=');
        buf.push(self.ord_status as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_SECURITY_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.security_id.as_bytes());
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
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_OPEN_CLOSE).as_bytes());
        buf.push(b'=');
        buf.push(self.open_close as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_EXEC_TYPE).as_bytes());
        buf.push(b'=');
        buf.push(self.exec_type as u8);
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_LEAVES_QTY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.leaves_qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_SECURITY_TYPE).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.security_type.as_bytes());
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

        buf.extend_from_slice(itoa_buf.format(TAG_MATURITY_DATE).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(self.maturity_date.as_bytes());
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut cl_ord_id = None;
        let mut cum_qty = None;
        let mut exec_id = None;
        let mut exec_trans_type = None;
        let mut order_id = None;
        let mut order_qty = None;
        let mut ord_status = None;
        let mut security_id = None;
        let mut side = None;
        let mut symbol = None;
        let mut open_close = None;
        let mut exec_type = None;
        let mut leaves_qty = None;
        let mut security_type = None;
        let mut put_or_call = None;
        let mut strike_price = None;
        let mut customer_or_firm = None;
        let mut maturity_date = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_CUM_QTY => {
                    cum_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_EXEC_ID => {
                    exec_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_EXEC_TRANS_TYPE => {
                    exec_trans_type = value
                        .first()
                        .copied()
                        .and_then(|b| ExecTransType::try_from(b).ok());
                }
                TAG_ORDER_ID => {
                    order_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_ORDER_QTY => {
                    order_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORD_STATUS => {
                    ord_status = value
                        .first()
                        .copied()
                        .and_then(|b| OrdStatus::try_from(b).ok());
                }
                TAG_SECURITY_ID => {
                    security_id = from_utf8(value).ok().map(str::to_owned);
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
                TAG_EXEC_TYPE => {
                    exec_type = value
                        .first()
                        .copied()
                        .and_then(|b| ExecType::try_from(b).ok());
                }
                TAG_LEAVES_QTY => {
                    leaves_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
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
                TAG_MATURITY_DATE => {
                    maturity_date = from_utf8(value).ok().map(str::to_owned);
                }
                _ => {}
            }
        }

        Ok(ExecutionReport {
            cl_ord_id: cl_ord_id.ok_or("Missing ClOrdID")?,
            cum_qty: cum_qty.ok_or("Missing CumQty")?,
            exec_id: exec_id.ok_or("Missing ExecID")?,
            exec_trans_type: exec_trans_type.ok_or("Missing ExecTransType")?,
            order_id: order_id.ok_or("Missing OrderID")?,
            order_qty: order_qty.ok_or("Missing OrderQty")?,
            ord_status: ord_status.ok_or("Missing OrdStatus")?,
            security_id: security_id.ok_or("Missing SecurityID")?,
            side: side.ok_or("Missing Side")?,
            symbol: symbol.ok_or("Missing Symbol")?,
            open_close: open_close.ok_or("Missing OpenClose")?,
            exec_type: exec_type.ok_or("Missing ExecType")?,
            leaves_qty: leaves_qty.ok_or("Missing LeavesQty")?,
            security_type: security_type.ok_or("Missing SecurityType")?,
            put_or_call: put_or_call.ok_or("Missing PutOrCall")?,
            strike_price: strike_price.ok_or("Missing StrikePrice")?,
            customer_or_firm: customer_or_firm.ok_or("Missing CustomerOrFirm")?,
            maturity_date: maturity_date.ok_or("Missing MaturityDate")?,
        })
    }
}
