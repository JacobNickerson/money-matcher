use crate::lob::{
    order::{Order, OrderSide, OrderType},
    types::{OrderId, Price, Timestamp},
};
use mio::{Token, net::TcpStream};
use netlib::fix_core::messages::execution_report::ExecutionReport;
use netlib::fix_core::{
    helpers::{convert_timestamp, extract_message, write_fix_message},
    messages::FIX_MESSAGE_TYPE_NEW_ORDER,
};
use netlib::fix_core::{iterator::FixIterator, messages::FixMessage};
use ringbuf::{HeapProd, traits::*};
use std::{
    io::{Read, Write},
    str::from_utf8,
};

const WAKE: Token = Token(1);
const MAX_BUFFER_SIZE: usize = 2048;

pub struct Session {
    token: Token,
    pub(crate) stream: TcpStream,
    read_buffer: Vec<u8>,
    pub(crate) write_buffer: Vec<u8>,
    tmp: [u8; MAX_BUFFER_SIZE],
    tmp_end: usize,
}
pub enum FIXRequest {
    Order(Token, Order),
}
pub enum FIXReply {
    ExecutionReport(Token, ExecutionReport),
}

impl Session {
    pub fn new(token: Token, stream: TcpStream) -> Self {
        Self {
            token,
            stream,
            read_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            write_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            tmp: [0u8; MAX_BUFFER_SIZE],
            tmp_end: 0,
        }
    }

    pub fn poll(&mut self, tx: &mut HeapProd<FIXRequest>) -> Result<(), &'static str> {
        loop {
            if self.tmp_end >= self.tmp.len() {
                break;
            }

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

        if self.read_buffer.len() + self.tmp_end > MAX_BUFFER_SIZE {
            self.tmp_end = 0;
            return Err("message too large");
        }

        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;

        self.process_messages(tx);

        Ok(())
    }

    fn process_messages(&mut self, tx: &mut HeapProd<FIXRequest>) {
        while let Some(msg) = extract_message(&mut self.read_buffer) {
            let mut msg_type = None;

            for (tag, value) in FixIterator::new(&msg) {
                if tag == b"35" {
                    msg_type = Some(value);
                    break;
                }
            }

            match msg_type {
                Some(FIX_MESSAGE_TYPE_NEW_ORDER) => self.handle_new_order(&msg, tx),
                _ => continue,
            };
        }
    }

    fn handle_new_order(
        &mut self,
        msg: &[u8],
        tx: &mut HeapProd<FIXRequest>,
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

        tx.try_push(FIXRequest::Order(self.token, order))
            .map_err(|_| "LOB queue full")?;

        Ok(())
    }

    pub fn handle_reply<T>(&mut self, reply: T)
    where
        T: FixMessage,
    {
        let sender_comp_id = "ENGINE01".to_string();
        let target_comp_id = "CLIENT01".to_string();

        let msg = write_fix_message(
            T::MESSAGE_TYPE,
            &1_u32,
            &sender_comp_id,
            &target_comp_id,
            &reply.as_bytes(),
        );

        self.write_buffer.extend_from_slice(&msg);
    }

    pub fn send_replies(&mut self) {
        let len: usize = self.stream.write(&self.write_buffer).unwrap();
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

    fn make_session() -> (Session, ringbuf::HeapRb<FIXRequest>) {
        let queue = HeapRb::<FIXRequest>::new(8);
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
            FIXRequest::Order(token, o1) => {
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
