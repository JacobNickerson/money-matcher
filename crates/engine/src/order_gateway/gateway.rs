use crate::lob::order::Order;
use ringbuf::{HeapCons, HeapProd, traits::*};
use std::mem::size_of;

pub const fn batch_size<T>() -> usize {
    let elem_size = size_of::<T>();
    const BATCH_BYTE_SIZE: usize = 128;
    let n = BATCH_BYTE_SIZE / elem_size;
    if n == 0 { 1 } else { n }
}

struct OrderMerger {
    synthetic_orders: HeapCons<Order>,
    user_orders: HeapCons<Order>,
    output: HeapProd<Order>,
    synthetic_buffer: Vec<Order>,
    user_buffer: Vec<Order>,
}
impl OrderMerger {
    pub fn new(
        synthetic_orders: HeapCons<Order>,
        user_orders: HeapCons<Order>,
        output: HeapProd<Order>,
    ) -> Self {
        Self {
            synthetic_orders,
            user_orders,
            output,
            synthetic_buffer: vec![Order::default(); batch_size::<Order>()],
            user_buffer: vec![Order::default(); batch_size::<Order>()],
        }
    }
    pub fn process_batch(&mut self) {
        let synth_count = self.synthetic_orders.pop_slice(&mut self.synthetic_buffer);
        let user_count = self.user_orders.pop_slice(&mut self.user_buffer);
    }
}
