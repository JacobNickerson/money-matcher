use crate::lob::{
    order::{Order, OrderSide, OrderType},
    types::{OrderId, Price, Timestamp},
};
use mio::net::TcpStream;
use netlib::fix_core::helpers::{calculate_checksum, convert_timestamp};
use netlib::fix_core::iterator::FixIterator;
use ringbuf::{HeapProd, traits::*};
use std::{io::Read, str::from_utf8};

pub struct Session {
    stream: TcpStream,
    buffer: Vec<u8>,
    tmp: [u8; 4096],
    tmp_end: usize,
}
pub enum FIXCommand {
    Order(Order),
}

impl Session {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
            tmp: [0u8; 4096],
            tmp_end: 0,
        }
    }

    pub fn poll(&mut self, tx: &mut HeapProd<Order>) -> Result<(), &'static str> {
        loop {
            match self.stream.read(&mut self.tmp[self.tmp_end..]) {
                Ok(0) => {
                    return Err("Peer closed connection");
                }
                Ok(n) => {
                    self.tmp_end += n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err("Error reading from stream");
                }
            }
        }
        self.buffer.extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;

        while let Some(msg) = self.extract_message() {
            let mut msg_type = None;

            for (tag, value) in FixIterator::new(&msg) {
                if tag == b"35" {
                    msg_type = Some(value);
                    break;
                }
            }

            match msg_type {
                Some(b"D") => self.handle_new_order(&msg, tx),
                _ => Ok(()),
            };
        }
        Ok(())
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

    fn handle_new_order(
        &mut self,
        msg: &[u8],
        tx: &mut HeapProd<Order>,
    ) -> Result<(), &'static str> {
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

        let qty: u64 = qty.ok_or("Missing 38")?.into();
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

        tx.try_push(order)
            .map_err(|_| "LOB queue full")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mio::net::TcpListener;
    use netlib::fix_core::helpers::write_trailer;
    use ringbuf::{HeapCons, HeapRb, traits::*};
    use std::net::SocketAddr;

    fn make_session() -> (Session, ringbuf::HeapRb<Order>) {
        let queue = HeapRb::<Order>::new(8);
        let listener_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(listener_addr).expect("err");
        let addr = listener.local_addr().expect("err");

        let _client = TcpStream::connect(addr).expect("err");
        let (server, _) = listener.accept().expect("err");

        (Session::new(server), queue)
    }

    #[test]
    fn test_handle_new_order_limit_bid() {
        let (mut session, q) = make_session();
        let (mut tx, mut rx) = q.split();

        let msg = b"8=FIX.4.2\x019=177\x0135=D\x0134=1\x0149=CLIENT01\x0152=20260223-16:56:36.513\x0156=ENGINE01\x0111=1\x0138=10\x0140=2\x0144=666\x0154=1\x0160=20260223-16:56:36.510\x0110=092\x01";

        session.handle_new_order(msg, &mut tx).expect("err");

        let o1 = rx.try_pop().expect("err");
        assert_eq!(o1.order_id, 1);
        assert_eq!(o1.side, OrderSide::Bid);
        assert_eq!(
            o1.kind,
            OrderType::Limit {
                qty: 10,
                price: 666
            }
        );
    }

    #[test]
    fn test_extract_message_valid_checksum() {
        let (mut session, _rx1) = make_session();

        let body = b"35=D\x01";
        let body_len = body.len();

        let mut msg = Vec::new();
        msg.extend_from_slice(b"8=FIX.4.2\x01");
        msg.extend_from_slice(b"9=");
        msg.extend_from_slice(body_len.to_string().as_bytes());
        msg.push(0x01);
        msg.extend_from_slice(body);

        write_trailer(&mut msg);

        session.buffer.extend_from_slice(&msg);

        let out = session.extract_message().expect("err");
        assert_eq!(out, msg);
    }

    #[test]
    fn test_extract_message_checksum_mismatch() {
        let (mut session, _rx1) = make_session();

        let msg = b"8=FIX.4.2\x019=5\x0135=D\x0110=000\x01";
        session.buffer.extend_from_slice(msg);

        assert!(session.extract_message().is_none());
    }
}
