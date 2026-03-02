use crate::fix_core::{
    helpers::{get_maturity_month_year, get_timestamp},
    messages::FixMessage,
};

pub struct NewOrder {
    pub cl_ord_id: u64,
    pub handl_inst: u8,
    pub qty: u32,
    pub ord_type: u8,
    pub price: u32,
    pub side: u8,
    pub symbol: String,
    pub open_close: u8,
    pub security_type: String,
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

impl FixMessage for NewOrder {
    const MESSAGE_TYPE: &'static [u8] = b"D";

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
        buf.extend_from_slice(self.symbol.as_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_order_initial_state() {
        let o = NewOrder::new(
            1,
            12,
            123,
            4,
            12345,
            1,
            "str1".to_string(),
            2,
            "str2".to_string(),
        );

        assert_eq!(o.cl_ord_id, 1);
        assert_eq!(o.handl_inst, 12);
        assert_eq!(o.qty, 123);
        assert_eq!(o.ord_type, 4);
        assert_eq!(o.price, 12345);
        assert_eq!(o.side, 1);
        assert_eq!(o.symbol, "str1");
        assert_eq!(o.open_close, 2);
        assert_eq!(o.security_type, "str2");
    }

    #[test]
    fn test_into_bytes_field_values() {
        let o = NewOrder::new(
            1,
            12,
            123,
            4,
            12345,
            1,
            "str1".to_string(),
            2,
            "str2".to_string(),
        );

        let b = o.as_bytes();
        let s = String::from_utf8_lossy(&b);

        assert!(s.contains("11=1"));
        assert!(s.contains("21=12"));
        assert!(s.contains("38=123"));
        assert!(s.contains("40=4"));
        assert!(s.contains("44=12345"));
        assert!(s.contains("54=1"));
        assert!(s.contains("55=str1"));
        assert!(s.contains("77=2"));
        assert!(s.contains("201=1"));
        assert!(s.contains("202=10"));
        assert!(s.contains("204=0"));
        assert!(s.contains("205=10"));
    }
}
