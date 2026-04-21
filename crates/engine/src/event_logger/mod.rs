use rkyv::{
    rancor::Error as RancorError, to_bytes, util::AlignedVec
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    vec::Vec,
};
use mm_core::lob_core::market_orders::Order;

pub struct BinaryLogger {
    buffer: Vec<u8>,
    buf_write: BufWriter<File>,
    batch_size: usize,
    batches_queued: usize,
}
impl BinaryLogger {
    pub fn new(path: &str, batch_size: usize) -> io::Result<Self> {
        let file = File::create(path)?;
    	Ok(Self {
            buffer: Vec::with_capacity(batch_size*size_of::<Order>()),
            buf_write: BufWriter::with_capacity(16*1024*1024, file), // TODO: Tune the size of this
            batch_size,
            batches_queued: 0,
    	})
    }
    pub fn log_order(&mut self, order: Order) {
        let bytes: AlignedVec = to_bytes::<RancorError>(&order).expect("binarylogger: failed to serialize order");
        self.buffer.extend_from_slice(bytes.as_ref());
        self.batches_queued += 1;
        if self.batches_queued == self.batch_size {
            self.buf_write.write_all(&self.buffer).expect("binarylogger: bufwriter failed to write");
        }
    }
}
