use netlib::fix_core::iterator::FixIterator;
use netlib::fix_core::messages::{calculate_checksum, print_message};
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
                    _ => {}
                }
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

    fn handle_new_order(&mut self, msg: &Vec<u8>) {
        for (tag, value) in FixIterator::new(msg) {
            println!(
                "{} = {}",
                String::from_utf8_lossy(&tag),
                String::from_utf8_lossy(&value)
            );
        }
    }
}
