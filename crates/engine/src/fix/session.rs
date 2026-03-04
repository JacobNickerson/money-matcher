use crate::lob::{
    order::{Order, OrderSide, OrderType},
    types::{OrderId, Price, Timestamp},
};
use mio::{Token, net::TcpStream};
use netlib::fix_core::iterator::FixIterator;
use netlib::fix_core::messages::execution_report::ExecutionReport;
use netlib::fix_core::{
    helpers::{convert_timestamp, extract_message, print_message, write_fix_message},
    messages::FIX_MESSAGE_TYPE_NEW_ORDER,
};
use ringbuf::{HeapProd, traits::*};
use std::{
    io::{Read, Write},
    str::from_utf8,
};

pub struct Session {
    token: Token,
    pub(crate) stream: TcpStream,
    read_buffer: Vec<u8>,
    pub(crate) write_buffer: Vec<u8>,
    tmp: [u8; 4096],
    tmp_end: usize,
}
pub enum FIXCommand {
    Order(Token, Order),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FIXReply {
    pub token: Token,
    pub data: ExecutionReport,
}

const WAKE: Token = Token(1);

impl Session {
    pub fn new(token: Token, stream: TcpStream) -> Self {
        Self {
            token,
            stream,
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            tmp: [0u8; 4096],
            tmp_end: 0,
        }
    }

    pub fn poll(&mut self, tx: &mut HeapProd<FIXCommand>) -> Result<(), &'static str> {
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
        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;

        while let Some(msg) = extract_message(&mut self.read_buffer) {
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

    fn handle_new_order(
        &mut self,
        msg: &[u8],
        tx: &mut HeapProd<FIXCommand>,
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

        tx.try_push(FIXCommand::Order(self.token, order))
            .map_err(|_| "LOB queue full")?;

        Ok(())
    }

    pub fn handle_reply(&mut self, report: ExecutionReport) {
        println!("IN HANDLE REPLY");
        let sender_comp_id = "ENGINE01".to_string();
        let target_comp_id = "CLIENT01".to_string();
        write_fix_message(
            &mut self.write_buffer,
            FIX_MESSAGE_TYPE_NEW_ORDER,
            &1_u32,
            &sender_comp_id,
            &target_comp_id,
            &report.as_bytes(),
        );
    }

    pub fn flush(&mut self) {
        println!("IN FLUSH");
        let len = self.stream.write(&self.write_buffer).unwrap();
        println!("SENT");
        print_message(&self.write_buffer);
        self.write_buffer.drain(..len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mio::net::TcpListener;
    use netlib::fix_core::helpers::write_trailer;
    use ringbuf::{HeapCons, HeapRb, traits::*};
    use std::net::SocketAddr;

    fn make_session() -> (Session, ringbuf::HeapRb<FIXCommand>) {
        let queue = HeapRb::<FIXCommand>::new(8);
        let listener_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(listener_addr).expect("err");
        let addr = listener.local_addr().expect("err");

        let _client = TcpStream::connect(addr).expect("err");
        let (server, _) = listener.accept().expect("err");

        (Session::new(Token(0), server), queue)
    }

    #[test]
    fn test_handle_new_order_limit_bid() {
        let (mut session, q) = make_session();
        let (mut tx, mut rx) = q.split();

        let msg = b"8=FIX.4.2\x019=177\x0135=D\x0134=1\x0149=CLIENT01\x0152=20260223-16:56:36.513\x0156=ENGINE01\x0111=1\x0138=10\x0140=2\x0144=666\x0154=1\x0160=20260223-16:56:36.510\x0110=092\x01";

        session.handle_new_order(msg, &mut tx).expect("err");

        let cmd = rx.try_pop().expect("err");
        match cmd {
            FIXCommand::Order(token, o1) => {
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
        }
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

        session.read_buffer.extend_from_slice(&msg);

        let out = extract_message(&mut session.read_buffer).expect("err");
        assert_eq!(out, msg);
    }

    #[test]
    fn test_extract_message_checksum_mismatch() {
        let (mut session, _rx1) = make_session();

        let msg = b"8=FIX.4.2\x019=5\x0135=D\x0110=000\x01";
        session.read_buffer.extend_from_slice(msg);

        assert!(extract_message(&mut session.read_buffer).is_none());
    }
}
