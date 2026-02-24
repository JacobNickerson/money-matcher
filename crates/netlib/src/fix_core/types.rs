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

pub struct FixFrame {
    pub msg_type: &'static [u8],
    pub body: Vec<u8>,
}
