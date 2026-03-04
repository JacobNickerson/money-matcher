use bytes::BytesMut;
use netlib::{
    itch_core::messages::{
        ItchEvent, ITCH_MESSAGE_TYPE_ADD_ORDER, ITCH_MESSAGE_TYPE_ORDER_CANCEL, ITCH_MESSAGE_TYPE_ORDER_DELETE,
        ITCH_MESSAGE_TYPE_ORDER_EXECUTED, ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE,
        ITCH_MESSAGE_TYPE_ORDER_REPLACE, ITCH_MESSAGE_TYPE_TEST_BENCHMARK, add_order::AddOrder,
        order_cancel::OrderCancel, order_delete::OrderDelete, order_executed::OrderExecuted,
        order_executed_with_price::OrderExecutedWithPrice, order_replace::OrderReplace,
        test_benchmark::TestBenchmark,
    },
    moldudp64_core::types::Header,
};
use nexus_queue::{Full, spsc};
use std::net::UdpSocket;
use zerocopy::FromBytes;

pub struct ReceiverHandler {
    socket: UdpSocket,
    output: spsc::Producer<ItchEvent>,
}

macro_rules! itch {
    ($type:expr, $data:expr, { $($type_const:path => $enum:ident ($struct:ty)),* $(,)? }) => {
        match $type {
            $(
                $type_const => <$struct>::read_from_prefix($data).ok().map(|(m, _)| ItchEvent::$enum(m)),
            )*
            _ => None,
        }
    };
}

impl ReceiverHandler {
    pub fn new(output: spsc::Producer<ItchEvent>, socket: UdpSocket) -> Self {
        Self { socket, output }
    }

    pub fn run(mut self) {
        let mut buf = BytesMut::with_capacity(2048);

        loop {
            buf.resize(2048, 0);

            let (len, _) = match self.socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(_e) => continue,
            };

            let bytes = buf.split_to(len).freeze();
            self.handle_packet(&bytes);
        }
    }

    fn handle_packet(&mut self, bytes: &[u8]) {
        let len: usize = bytes.len();
        if len < 20 {
            return;
        }

        let header = match Header::read_from_prefix(bytes) {
            Ok(v) => v.0,
            Err(_) => return,
        };

        let mc = u16::from_be_bytes(header.message_count) as usize;
        let mut offset = 20;

        for _ in 0..mc {
            if !self.handle_message(bytes, len, &mut offset) {
                break;
            }
        }
    }

    fn handle_message(&mut self, bytes: &[u8], len: usize, offset: &mut usize) -> bool {
        if *offset + 2 > len {
            return false;
        }

        let ml = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]) as usize;
        *offset += 2;

        if *offset + ml > len {
            return false;
        }

        let message_data = &bytes[*offset..*offset + ml];
        *offset += ml;

        if message_data.is_empty() {
            return true;
        }

        let event = match Self::parse_event(message_data) {
            Some(v) => v,
            None => return true,
        };

        self.push_event(event);
        true
    }

    fn parse_event(message_data: &[u8]) -> Option<ItchEvent> {
        if message_data.is_empty() {
            return None;
        }

        let message_type = message_data[0];

        itch!(message_type, message_data, {
            ITCH_MESSAGE_TYPE_ADD_ORDER => AddOrder(AddOrder),
            ITCH_MESSAGE_TYPE_ORDER_CANCEL => OrderCancel(OrderCancel),
            ITCH_MESSAGE_TYPE_ORDER_DELETE => OrderDelete(OrderDelete),
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED => OrderExecuted(OrderExecuted),
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE => OrderExecutedWithPrice(OrderExecutedWithPrice),
            ITCH_MESSAGE_TYPE_ORDER_REPLACE => OrderReplace(OrderReplace),
            ITCH_MESSAGE_TYPE_TEST_BENCHMARK => TestBenchmark(TestBenchmark),
        })
    }

    fn push_event(&mut self, mut event: ItchEvent) {
        loop {
            match self.output.push(event) {
                Ok(_) => break,
                Err(Full(e)) => {
                    event = e;
                    std::hint::spin_loop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::IntoBytes;

    fn make_handler() -> (ReceiverHandler, spsc::Consumer<ItchEvent>) {
        let (tx, rx) = spsc::ring_buffer::<ItchEvent>(8);
        let socket = UdpSocket::bind("127.0.0.1:0").expect("err");
        (ReceiverHandler::new(tx, socket), rx)
    }

    #[test]
    fn test_parse_event_test_benchmark() {
        let msg = TestBenchmark::new(123);
        let bytes = msg.as_bytes();

        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::TestBenchmark(v) => {
                assert_eq!(v.timestamp, msg.timestamp);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_add_order() {
        let mut stock = [b' '; 8];
        stock[..4].copy_from_slice(b"TEST");

        let msg = AddOrder::new(1, 12, 123, b'B', 10, stock, 99.into());
        let bytes = msg.as_bytes();

        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::AddOrder(v) => {
                assert_eq!(v.shares.get(), 10);
                assert_eq!(v.price.get(), 990000);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_cancel() {
        let msg = OrderCancel::new(1, 1000, 5000, 10);
        let bytes = msg.as_bytes();
        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::OrderCancel(v) => {
                assert_eq!(v.stock_locate.get(), 1);
                assert_eq!(v.order_reference_number.get(), 5000);
                assert_eq!(v.canceled_shares.get(), 10);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_delete() {
        let msg = OrderDelete::new(1, 1000, 5000);
        let bytes = msg.as_bytes();
        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::OrderDelete(v) => {
                assert_eq!(v.stock_locate.get(), 1);
                assert_eq!(v.order_reference_number.get(), 5000);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_executed_with_price() {
        let msg = OrderExecutedWithPrice::new(1, 1000, 5000, 10, 9999, b'Y', 100.0);
        let bytes = msg.as_bytes();
        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::OrderExecutedWithPrice(v) => {
                assert_eq!(v.order_reference_number.get(), 5000);
                assert_eq!(v.executed_shares.get(), 10);
                assert_eq!(v.execution_price.get(), 1000000);
                assert_eq!(v.printable, b'Y');
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_executed() {
        let msg = OrderExecuted::new(1, 1000, 5000, 100, 9999);
        let bytes = msg.as_bytes();
        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::OrderExecuted(v) => {
                assert_eq!(v.order_reference_number.get(), 5000);
                assert_eq!(v.executed_shares.get(), 100);
                assert_eq!(v.match_number.get(), 9999);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_replace() {
        let msg = OrderReplace::new(1, 1000, 5000, 5001, 20, 100.0);
        let bytes = msg.as_bytes();
        let event = ReceiverHandler::parse_event(bytes).expect("err");

        match event {
            ItchEvent::OrderReplace(v) => {
                assert_eq!(v.original_order_reference_number.get(), 5000);
                assert_eq!(v.new_order_reference_number.get(), 5001);
                assert_eq!(v.shares.get(), 20);
                assert_eq!(v.price.get(), 1000000);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_handle_message_updates_offset() {
        let (mut h, _rx) = make_handler();

        let msg = TestBenchmark::new(123);
        let bytes = msg.as_bytes();

        let mut buf = Vec::new();
        buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(bytes);

        let mut offset = 0;
        let ok = h.handle_message(&buf, buf.len(), &mut offset);

        assert!(ok);
        assert_eq!(offset, buf.len());
    }

    #[test]
    fn test_handle_packet_multiple_messages() {
        let (mut h, mut rx) = make_handler();

        let msg1 = TestBenchmark::new(1);
        let msg2 = TestBenchmark::new(2);

        let bytes1 = msg1.as_bytes();
        let bytes2 = msg2.as_bytes();

        let header = Header {
            session_id: [b'A'; 10],
            sequence_number: 1u64.to_be_bytes(),
            message_count: (2u16).to_be_bytes(),
        };

        let mut packet = Vec::new();
        packet.extend_from_slice(header.as_bytes());

        packet.extend_from_slice(&(bytes1.len() as u16).to_be_bytes());
        packet.extend_from_slice(bytes1);

        packet.extend_from_slice(&(bytes2.len() as u16).to_be_bytes());
        packet.extend_from_slice(bytes2);

        h.handle_packet(&packet);

        assert!(matches!(
            rx.pop().expect("err"),
            ItchEvent::TestBenchmark(_)
        ));
        assert!(matches!(
            rx.pop().expect("err"),
            ItchEvent::TestBenchmark(_)
        ));
    }

    #[test]
    fn test_handle_packet_ignores_short_packet() {
        let (mut h, mut rx) = make_handler();

        let buf = [0u8; 10];
        h.handle_packet(&buf);

        assert!(rx.pop().is_none());
    }
}
