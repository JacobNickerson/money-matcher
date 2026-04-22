use mm_core::lob_core::market_orders::Order;
use rand_distr::num_traits::ToBytes;
use rkyv::util::AlignedVec;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    vec::Vec,
};

pub struct BinaryLogger {
    writer: BufWriter<File>,
}
impl BinaryLogger {
    pub fn new(path: &str, batch_size: usize) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }
    pub fn log_order(&mut self, order: &Order) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(order)?;
        let len = bytes.len() as u8;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&bytes)?;
        Ok(())
    }
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
