use netlib::fix_core::helpers::{calculate_checksum, convert_timestamp};
use netlib::fix_core::iterator::FixIterator;
use netlib::fix_core::types::NewOrder;
use nexus_queue::mpsc::Producer;
use std::{io::Read, net::TcpStream, str::from_utf8};

use crate::lob::types::Timestamp;
use crate::lob::{
    order::{Order, OrderSide, OrderType},
    types::{OrderId, Price},
};

pub struct Session {
    stream: TcpStream,
    lob_tx: Producer<FIXCommand>,
    buffer: Vec<u8>,
}

impl Session {
    pub fn new(stream: TcpStream, lob_tx: Producer<FIXCommand>) -> Self {
        Self {
            stream,
            lob_tx,
            buffer: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let mut tmp = [0u8; 4096];

        loop {
            let n = match self.stream.read(&mut tmp) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };

            self.buffer.extend_from_slice(&tmp[..n]);

            while let Some(msg) = self.extract_message() {
                let mut msg_type = None;

                for (tag, value) in FixIterator::new(&msg) {
                    if tag == b"35" {
                        msg_type = Some(value);
                        break;
                    }
                }

                match msg_type {
                    Some(b"D") => self.handle_new_order(&msg),
                    _ => Ok(()),
                };
            }
        }
    }

    fn extract_message(&mut self) -> Option<Vec<u8>> {
        if !self.buffer.starts_with(b"8=FIX") {
            if let Some(position) = self.buffer.windows(5).position(|f| f == b"8=FIX") {
                self.buffer.drain(0..position);
            } else {
                self.buffer.clear();
            }

            return None;
        };

        let body_len_start = self.buffer.windows(2).position(|f| f == b"9=")?;
        let body_len_end = self.buffer[body_len_start..]
            .iter()
            .position(|&f| f == b'\x01')?
            + body_len_start;

        let body_len: usize = from_utf8(&self.buffer[body_len_start + 2..body_len_end])
            .ok()?
            .parse()
            .ok()?;

        let body_start = body_len_end + 1;
        let body_end = body_start + body_len;
        let total_len = body_end + 7;
        let recv_checksum: u32 = from_utf8(&self.buffer[body_end + 3..body_end + 6])
            .ok()?
            .parse()
            .ok()?;
        let checksum = calculate_checksum(&self.buffer[..body_end]);

        if recv_checksum != checksum {
            println!("Checksum mismatch {} {}", recv_checksum, checksum);
            return None;
        }

        if self.buffer.len() < total_len {
            return None;
        }

        Some(self.buffer.drain(0..total_len).collect())
    }

    fn handle_new_order(&mut self, msg: &[u8]) -> Result<(), &'static str> {
        let mut cl_ord_id: Option<OrderId> = None;
        let mut qty: Option<u32> = None;
        let mut price: Option<Price> = None;
        let mut side: Option<OrderSide> = None;
        let mut ord_type: Option<u8> = None;
        let mut timestamp: Option<Timestamp> = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                b"11" => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                b"38" => {
                    qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                b"44" => {
                    price = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                b"54" => {
                    side = match value {
                        b"1" => Some(OrderSide::Bid),
                        b"2" => Some(OrderSide::Ask),
                        _ => return Err("Invalid 54"),
                    };
                }
                b"40" => {
                    ord_type = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                b"60" => {
                    timestamp = convert_timestamp(value);
                }
                _ => {}
            }
        }

        let qty = qty.ok_or("Missing 38")?;
        let kind = match ord_type.ok_or("Missing 40")? {
            1 => OrderType::Market { qty },
            2 => OrderType::Limit {
                qty,
                price: price.ok_or("Missing 44")?,
            },
            _ => return Err("Unsupported 40"),
        };

        let order = Order::new(
            cl_ord_id.ok_or("Missing 11")?,
            side.ok_or("Missing 54")?,
            timestamp.ok_or("Missing 60")?,
            kind,
        );

        self.lob_tx
            .push(FIXCommand::Order(order))
            .map_err(|_| "LOB queue full")?;

        Ok(())
    }
}
pub enum FIXCommand {
    Order(Order),
}
