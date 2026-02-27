use crate::lob::order::Order;
use ringbuf::{HeapCons, HeapProd, traits::*};
use std::mem::size_of;

/// Determines the batch size for processing orders based on the size of the struct
/// Batches are sized to fit within a certain byte size (e.g., 128 bytes) to optimize cache usage and reduce overhead
pub const fn batch_size<T>() -> usize {
    let elem_size = size_of::<T>();
    const BATCH_BYTE_SIZE: usize = 128;
    let n = BATCH_BYTE_SIZE / elem_size;
    if n == 0 { 1 } else { n }
}

/// Wrapper around a Vec to provide a simple interface for handling a FIFO buffer for orders
struct MergeBuffer<T: Copy> {
    buf: Vec<T>,
    count: usize,
    start: usize,
    end: usize,
}
impl<T: Copy> MergeBuffer<T> {
    pub fn new(buf: Vec<T>) -> Self {
        Self {
            buf,
            count: 0,
            start: 0,
            end: 0,
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let item = self.buf[self.start];
            self.start += 1;
            self.count -= 1;
            Some(item)
        }
    }
    pub fn peek(&self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            Some(self.buf[self.start])
        }
    }
    pub fn slide(&mut self) {
        if self.end == self.buf.len() && self.start > 0 {
            self.buf.copy_within(self.start..self.end, 0);
            self.end -= self.start;
            self.start = 0;
        }
    }
    pub fn empty(&mut self) -> bool {
        self.count == 0
    }
    pub fn pop_slice(&mut self, output: &mut HeapProd<T>) {
        let pushed = output.push_slice(&self.buf[self.start..(self.start + self.count)]);
        self.start += pushed;
        self.count -= pushed;
        self.slide();
    }
}

/// Merges two streams of orders (synthetic and user) into a single output stream while maintaining chronological order based on timestamps
pub struct OrderMerger {
    synthetic_orders: HeapCons<Order>,
    user_orders: HeapCons<Order>,
    output: HeapProd<Order>,
    synthetic_buffer: MergeBuffer<Order>,
    user_buffer: MergeBuffer<Order>,
}
impl OrderMerger {
    pub fn new(
        synthetic_orders: HeapCons<Order>,
        user_orders: HeapCons<Order>,
        output: HeapProd<Order>,
        internal_buffer_size: usize,
    ) -> Self {
        Self {
            synthetic_orders,
            user_orders,
            output,
            synthetic_buffer: MergeBuffer::<Order>::new(vec![
                Order::default();
                internal_buffer_size
            ]),
            user_buffer: MergeBuffer::<Order>::new(vec![Order::default(); internal_buffer_size]),
        }
    }
    /// Reads a batch of orders from both synthetic and user streams into internal buffers for processing
    pub fn batch_read(&mut self) {
        let synth_order_count = self
            .synthetic_orders
            .pop_slice(&mut self.synthetic_buffer.buf[self.synthetic_buffer.end..]);
        let user_order_count = self
            .user_orders
            .pop_slice(&mut self.user_buffer.buf[self.user_buffer.end..]);
        self.synthetic_buffer.count += synth_order_count;
        self.user_buffer.count += user_order_count;
        self.synthetic_buffer.end += synth_order_count;
        self.user_buffer.end += user_order_count;
    }

    /// Merges the two internal buffers of orders into the output stream while maintaining chronological order based on timestamps
    pub fn process_batch(&mut self) {
        self.batch_read();
        if self.synthetic_buffer.empty() {
            self.user_buffer.pop_slice(&mut self.output);
            return;
        } else if self.user_buffer.empty() {
            self.synthetic_buffer.pop_slice(&mut self.output);
            return;
        }
        while let (Some(synth_order), Some(user_order)) =
            (self.synthetic_buffer.peek(), self.user_buffer.peek())
        {
            if synth_order.timestamp <= user_order.timestamp {
                match self.output.try_push(synth_order) {
                    Ok(()) => {
                        self.synthetic_buffer.start += 1;
                        self.synthetic_buffer.count -= 1;
                    }
                    Err(_) => break,
                }
            } else {
                match self.output.try_push(user_order) {
                    Ok(()) => {
                        self.user_buffer.start += 1;
                        self.user_buffer.count -= 1
                    }
                    Err(_) => break,
                }
            }
        }
        self.synthetic_buffer.slide();
        self.user_buffer.slide();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lob::order::{OrderSide, OrderType};
    use ringbuf::HeapRb;

    fn create_merger() -> (
        OrderMerger,
        HeapProd<Order>,
        HeapProd<Order>,
        HeapCons<Order>,
    ) {
        let synthetic_orders = HeapRb::<Order>::new(batch_size::<Order>() * 16);
        let (synth_prod, synth_cons) = synthetic_orders.split();
        let user_orders = HeapRb::<Order>::new(batch_size::<Order>() * 16);
        let (user_prod, user_cons) = user_orders.split();
        let output = HeapRb::<Order>::new(batch_size::<Order>() * 256);
        let (output_prod, output_cons) = output.split();
        (
            OrderMerger::new(
                synth_cons,
                user_cons,
                output_prod,
                batch_size::<Order>() * 32,
            ),
            synth_prod,
            user_prod,
            output_cons,
        )
    }

    #[test]
    fn test_typical_merge() {
        let (mut merger, mut synth_prod, mut user_prod, mut output_cons) = create_merger();
        for i in 0..10 {
            synth_prod
                .try_push(Order::new(2 * i, OrderSide::Ask, 2 * i, OrderType::Cancel))
                .unwrap();
            user_prod
                .try_push(Order::new(
                    2 * i + 1,
                    OrderSide::Ask,
                    2 * i + 1,
                    OrderType::Cancel,
                ))
                .unwrap();
        }
        merger.process_batch();
        merger.process_batch(); // Call twice to get last elements since process_batch() goes til one batch is empty
        let mut curr = output_cons.try_pop().unwrap().timestamp;
        let mut count = 1;
        while let Some(next) = output_cons.try_pop() {
            assert!(
                next.timestamp >= curr,
                "Timestamps are not in order: {} followed by {}",
                curr,
                next.timestamp
            );
            curr = next.timestamp;
            count += 1;
        }
        assert!(count == 20, "{} / 20 orders were processed", count)
    }

    #[test]
    fn test_multiple_merges() {
        let (mut merger, mut synth_prod, mut user_prod, mut output_cons) = create_merger();
        for i in 0..100 {
            synth_prod
                .try_push(Order::new(2 * i, OrderSide::Ask, 2 * i, OrderType::Cancel))
                .unwrap();
            user_prod
                .try_push(Order::new(
                    2 * i + 1,
                    OrderSide::Ask,
                    2 * i + 1,
                    OrderType::Cancel,
                ))
                .unwrap();
            if i % 10 == 9 {
                merger.process_batch();
            }
        }
        merger.process_batch();
        let mut curr = output_cons.try_pop().unwrap().timestamp;
        let mut count = 1;
        while let Some(next) = output_cons.try_pop() {
            assert!(
                next.timestamp >= curr,
                "Timestamps are not in order: {} followed by {}",
                curr,
                next.timestamp
            );
            curr = next.timestamp;
            count += 1;
        }
        assert!(count == 200, "{} / 200 orders were processed", count)
    }

    #[test]
    fn test_one_sided_flush() {
        let (mut merger, mut synth_prod, mut user_prod, mut output_cons) = create_merger();
        for i in 0..10 {
            synth_prod
                .try_push(Order::new(2 * i, OrderSide::Ask, 2 * i, OrderType::Cancel))
                .unwrap();
        }
        merger.process_batch();
        let mut curr = output_cons.try_pop().unwrap().timestamp;
        let mut count = 1;
        while let Some(next) = output_cons.try_pop() {
            assert!(
                next.timestamp >= curr,
                "Timestamps are not in order: {} followed by {}",
                curr,
                next.timestamp
            );
            curr = next.timestamp;
            count += 1;
        }
        assert!(count == 10, "{} / orders were flushed", count);
        for i in 10..20 {
            user_prod
                .try_push(Order::new(2 * i, OrderSide::Ask, 2 * i, OrderType::Cancel))
                .unwrap();
        }
        merger.process_batch();
        while let Some(next) = output_cons.try_pop() {
            assert!(
                next.timestamp >= curr,
                "Timestamps are not in order: {} followed by {}",
                curr,
                next.timestamp
            );
            curr = next.timestamp;
            count += 1;
        }
        assert!(count == 20, "{} / 20 orders were flushed", count);
    }

    #[test]
    fn test_empty_merge() {
        let (mut merger, _, _, mut output_cons) = create_merger();
        merger.process_batch();
        assert!(output_cons.try_pop().is_none());
    }
}
