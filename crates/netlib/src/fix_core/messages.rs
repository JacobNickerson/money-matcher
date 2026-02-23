use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    types::NewOrder,
};

pub trait IntoBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl NewOrder {
    pub fn new(
        cl_ord_id: u64,
        handl_inst: u8,
        qty: u32,
        ord_type: u8,
        price: u32,
        side: u8,
        symbol: String,
        open_close: u8,
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
impl IntoBytes for NewOrder {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"21=");
        buf.extend_from_slice(itoa_buf.format(self.handl_inst).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"38=");
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"40=");
        buf.extend_from_slice(itoa_buf.format(self.ord_type).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"44=");
        buf.extend_from_slice(itoa_buf.format(self.price).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"54=");
        buf.extend_from_slice(itoa_buf.format(self.side).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"55=");
        buf.extend_from_slice(&self.symbol.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"60=");
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"77=");
        buf.extend_from_slice(itoa_buf.format(self.open_close).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"77=");
        buf.extend_from_slice(itoa_buf.format(self.open_close).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"200=");
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"201=1\x01");
        buf.extend_from_slice(b"202=10\x01");
        buf.extend_from_slice(b"204=0\x01");
        buf.extend_from_slice(b"205=10\x01");

        buf
    }
}
