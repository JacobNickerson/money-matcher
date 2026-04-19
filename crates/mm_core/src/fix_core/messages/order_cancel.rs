use crate::{
    fix_core::{
        helpers::{convert_timestamp, get_timestamp, to_timestamp},
        iterator::FixIterator,
        messages::{
            FIXBusinessMessage, FIXMessage, TAG_CL_ORD_ID, TAG_ORDER_QTY, TAG_ORIG_CL_ORD_ID,
            TAG_TRANSACT_TIME,
        },
    },
    lob_core::market_orders::{Order, OrderSide, OrderType},
};
use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use std::str::from_utf8;
/// The Order Cancel Request message is used to cancel a regular or multi-leg order.
///
/// `MsgType = F`
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct OrderCancel {
    /// Maximum 20 characters. Any value exceeding 20 characters will be rejected.
    pub cl_ord_id: u64,
    /// Number of known open contracts.
    pub qty: u32,
    /// ClOrdID of the order to be canceled.
    pub orig_cl_ord_id: u64,
    pub transact_time: Option<String>,
}

impl FIXBusinessMessage for OrderCancel {
    fn to_order(self) -> Order {
        Order {
            client_id: self.orig_cl_ord_id,
            order_id: 0, // NOTE: This is set by simulator so the value doesn't matter
            side: OrderSide::Bid,
            timestamp: convert_timestamp(self.transact_time.expect("")).expect(""),
            kind: OrderType::Cancel,
        }
    }

    fn from_order(order: &Order) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        Ok(Self {
            cl_ord_id: order.order_id,
            qty: 0,
            orig_cl_ord_id: match order.kind {
                OrderType::Cancel => order.order_id,
                _ => return Err("Invalid order kind"),
            },
            transact_time: Some(to_timestamp(order.timestamp)),
        })
    }
}

impl FIXMessage for OrderCancel {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(itoa_buf.format(TAG_CL_ORD_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORDER_QTY).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_ORIG_CL_ORD_ID).as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.orig_cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(itoa_buf.format(TAG_TRANSACT_TIME).as_bytes());
        buf.push(b'=');
        if let Some(timestamp) = &self.transact_time {
            buf.extend_from_slice(timestamp.as_bytes());
        } else {
            buf.extend_from_slice(get_timestamp().as_bytes());
        }
        buf.push(0x01);

        buf
    }

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut cl_ord_id = None;
        let mut qty = None;
        let mut orig_cl_ord_id = None;
        let mut transact_time = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORDER_QTY => {
                    qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORIG_CL_ORD_ID => {
                    orig_cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_TRANSACT_TIME => transact_time = from_utf8(value).ok().map(str::to_owned),
                _ => {}
            }
        }

        Ok(OrderCancel {
            cl_ord_id: cl_ord_id.ok_or("Missing ClOrdID")?,
            qty: qty.ok_or("Missing Qty")?,
            orig_cl_ord_id: orig_cl_ord_id.ok_or("Missing OrigClOrdID")?,
            transact_time: Some(transact_time.ok_or("Missing TransactTime")?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_cancel_initial_state() {
        let o = OrderCancel {
            cl_ord_id: 1,
            qty: 123,
            orig_cl_ord_id: 456,
            transact_time: None,
        };

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.qty, 123);
        assert_eq!(o.orig_cl_ord_id, 456);
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = OrderCancel {
            cl_ord_id: 1,
            qty: 123,
            orig_cl_ord_id: 456,
            transact_time: None,
        };

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("38=123"));
        assert!(s.contains("41=456"));
    }
}
