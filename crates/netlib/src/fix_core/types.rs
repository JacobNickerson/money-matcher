pub struct NewOrder {
    pub cl_ord_id: u64,
    pub qty: u32,
    pub price: u32,
    pub side: u8,
    pub symbol: [u8; 3],
}
