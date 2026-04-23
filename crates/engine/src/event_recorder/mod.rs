use clap::ValueEnum;
use mm_core::lob_core::market_orders::Order;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

/// Enum denoting the type of a recorder. Used for selecting a Recorder using command-line args
#[derive(Clone, Copy, ValueEnum)]
pub enum RecorderType {
    Binary,
    Text,
}

/// Enum containing Recorders for dynamic selection of recorders
pub enum RecorderEnum {
    Binary(BinaryRecorder),
    Text(TextRecorder),
}
impl RecorderEnum {
    pub fn record_event(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            RecorderEnum::Binary(this) => this.record_event(order),
            RecorderEnum::Text(this) => this.record_event(order),
        }
    }
    pub fn shutdown(&mut self) -> io::Result<()> {
        match self {
            RecorderEnum::Binary(this) => this.shutdown(),
            RecorderEnum::Text(this) => this.shutdown(),
        }
    }
}

/// Trait that all Recorders must implement.
pub trait Recorder {
    /// Record an event
    fn record_event(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>>;
    /// Called when the Recorder is cleaning up, typically used to flush to file
    fn shutdown(&mut self) -> io::Result<()>;
}

/// Recorder that writes structs in a constant-size byte-packed format to file
pub struct BinaryRecorder {
    writer: BufWriter<File>,
    batch_size: usize,
    current: usize,
}
impl BinaryRecorder {
    pub fn new(path: &str, batch_size: usize) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            batch_size,
            current: 0,
        })
    }
}
impl Recorder for BinaryRecorder {
    fn record_event(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(&order.to_bytes())?;
        self.current += 1;
        if self.current == self.batch_size {
            self.current = 0;
            self.writer.flush()?;
        }
        Ok(())
    }
    fn shutdown(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// Recorder that writes structs in plaintext to file
pub struct TextRecorder {
    writer: BufWriter<File>,
    batch_size: usize,
    current: usize,
}
impl TextRecorder {
    pub fn new(path: &str, batch_size: usize) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            batch_size,
            current: 0,
        })
    }
}
impl Recorder for TextRecorder {
    fn record_event(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(format!("{:?}\n", order).as_bytes())?;
        self.current += 1;
        if self.current == self.batch_size {
            self.current = 0;
            self.writer.flush()?;
        }
        Ok(())
    }
    fn shutdown(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
