use zerocopy::IntoBytes;

use crate::fix_core::{
    helpers::get_timestamp,
    messages::{
        FIX_MESSAGE_TYPE_EXECUTION_REPORT, FixMessage, TAG_CL_ORD_ID, TAG_CUM_QTY,
        TAG_CUSTOMER_OR_FIRM, TAG_EXEC_ID, TAG_EXEC_TRANS_TYPE, TAG_EXEC_TYPE, TAG_LEAVES_QTY,
        TAG_MATURITY_DATE, TAG_OPEN_CLOSE, TAG_ORD_STATUS, TAG_ORDER_ID, TAG_ORDER_QTY,
        TAG_PUT_OR_CALL, TAG_SECURITY_ID, TAG_SECURITY_TYPE, TAG_SIDE, TAG_STRIKE_PRICE,
        TAG_SYMBOL, TAG_TRANSACT_TIME,
        types::{CustomerOrFirm, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side},
    },
};

/// The Execution Report message is used to:
/// • confirm the receipt of an order
/// • confirm changes to an existing order
/// • confirm cancelation of an existing order
/// • relay order status information
/// • relay fill information on working orders
/// • reject orders
/// • report trade busts or other post-trade corrections
///
/// MsgType = 8
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

impl FixMessage for ExecutionReport {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_EXECUTION_REPORT;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        // 11 - ClOrdID
        buf.extend_from_slice(TAG_CL_ORD_ID);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        // 14 - CumQty
        buf.extend_from_slice(TAG_CUM_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cum_qty).as_bytes());
        buf.push(0x01);

        // 17 - ExecID
        buf.extend_from_slice(TAG_EXEC_ID);
        buf.push(b'=');
        buf.extend_from_slice(self.exec_id.as_bytes());
        buf.push(0x01);

        // 20 - ExecTransType
        buf.extend_from_slice(TAG_EXEC_TRANS_TYPE);
        buf.push(b'=');
        buf.push(self.exec_trans_type as u8);
        buf.push(0x01);

        // 37 - OrderID
        buf.extend_from_slice(TAG_ORDER_ID);
        buf.push(b'=');
        buf.extend_from_slice(self.order_id.as_bytes());
        buf.push(0x01);

        // 38 - OrderQty
        buf.extend_from_slice(TAG_ORDER_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.order_qty).as_bytes());
        buf.push(0x01);

        // 39 - OrdStatus
        buf.extend_from_slice(TAG_ORD_STATUS);
        buf.push(b'=');
        buf.push(self.ord_status as u8);
        buf.push(0x01);

        // 48 - SecurityID
        buf.extend_from_slice(TAG_SECURITY_ID);
        buf.push(b'=');
        buf.extend_from_slice(self.security_id.as_bytes());
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

        // 150 - ExecType
        buf.extend_from_slice(TAG_EXEC_TYPE);
        buf.push(b'=');
        buf.push(self.exec_type as u8);
        buf.push(0x01);

        // 151 - LeavesQty
        buf.extend_from_slice(TAG_LEAVES_QTY);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.leaves_qty).as_bytes());
        buf.push(0x01);

        // 167 - SecurityType
        buf.extend_from_slice(TAG_SECURITY_TYPE);
        buf.push(b'=');
        buf.extend_from_slice(self.security_type.as_bytes());
        buf.push(0x01);

        // 201 - PutOrCall
        buf.extend_from_slice(TAG_PUT_OR_CALL);
        buf.push(b'=');
        buf.push(self.put_or_call as u8);
        buf.push(0x01);

        // 202 - StrikePrice
        buf.extend_from_slice(TAG_STRIKE_PRICE);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.strike_price).as_bytes());
        buf.push(0x01);

        // 204 - CustomerOrFirm
        buf.extend_from_slice(TAG_CUSTOMER_OR_FIRM);
        buf.push(b'=');
        buf.push(self.customer_or_firm as u8);
        buf.push(0x01);

        // 541 - MaturityDate
        buf.extend_from_slice(TAG_MATURITY_DATE);
        buf.push(b'=');
        buf.extend_from_slice(self.maturity_date.as_bytes());
        buf.push(0x01);

        buf
    }
}
