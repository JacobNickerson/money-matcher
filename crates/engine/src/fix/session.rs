use netlib::fix_core::iterator::FixIterator;
use netlib::fix_core::messages::calculate_checksum;
use std::{io::Read, net::TcpStream, str::from_utf8};

pub struct Session {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl Session {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
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

    fn handle_new_order(&mut self, msg: &Vec<u8>) -> Result<(), &str> {
        let mut cl_ord_id: Option<u64> = None;
        let mut qty: Option<u32> = None;
        let mut price: Option<u32> = None;
        let mut side: Option<u8> = None;
        let mut symbol: Option<String> = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                b"11" => {
                    cl_ord_id = from_utf8(value).ok().and_then(|f| f.parse().ok());
                }
                b"38" => {
                    qty = from_utf8(value).ok().and_then(|f| f.parse().ok());
                }
                b"44" => {
                    price = from_utf8(value).ok().and_then(|f| f.parse().ok());
                }
                b"54" => {
                    side = from_utf8(value).ok().and_then(|f| f.parse().ok());
                }
                b"55" => {
                    symbol = from_utf8(value).ok().and_then(|f| Some(f.to_owned()));
                }
                _ => {}
            }
        }

        println!(
            "Read New Order | cl_ord_id(11)={} | qty(38)={} | price(44)={} | side(54)={} | symbol(55)={}",
            cl_ord_id.ok_or("")?,
            qty.ok_or("")?,
            price.ok_or("")?,
            side.ok_or("")?,
            symbol.ok_or("")?
        );

        Ok(())
    }
}
