use bytes::{Bytes, BytesMut};
use netlib::moldudp64_core::types::{
    AddOrder, Header, ItchEvent, OrderExecutedMessage, TestBenchmark,
};
use nexus_queue::{Full, spsc};
use std::net::UdpSocket;
use zerocopy::FromBytes;
pub struct ReceiverHandler {
    socket: UdpSocket,
    output: spsc::Producer<ItchEvent>,
}

impl ReceiverHandler {
    pub fn new(output: spsc::Producer<ItchEvent>) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:8081").unwrap();
        Self { socket, output }
    }

    pub fn run(mut self) {
        let mut buf = BytesMut::with_capacity(2048);

        loop {
            buf.clear();
            buf.reserve(2048);
            unsafe { buf.set_len(2048) };

            let (len, _) = self.socket.recv_from(&mut buf).unwrap();
            unsafe { buf.set_len(len) };

            let bytes = buf.split_to(len).freeze();

            let len: usize = bytes.len();
            if len < 20 {
                continue;
            }

            let header = match Header::read_from_prefix(&bytes) {
                Ok(v) => v.0,
                Err(_) => continue,
            };

            let mc = u16::from_be_bytes(header.message_count) as usize;
            let mut offset = 20;

            for _ in 0..mc {
                if offset + 2 > len {
                    break;
                }

                let ml = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
                offset += 2;

                if offset + ml > len {
                    break;
                }

                let message_data = &bytes[offset..offset + ml];
                offset += ml;

                if message_data.is_empty() {
                    continue;
                }

                let message_type = message_data[0];

                let mut event: ItchEvent = match message_type {
                    b'b' => {
                        let (msg, _) = match TestBenchmark::read_from_prefix(message_data) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        ItchEvent::TestBenchmark(msg)
                    }
                    b'A' => {
                        let (msg, _) = match AddOrder::read_from_prefix(message_data) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        ItchEvent::AddOrder(msg)
                    }
                    b'E' => {
                        let (msg, _) = match OrderExecutedMessage::read_from_prefix(message_data) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        ItchEvent::OrderExecutedMessage(msg)
                    }
                    _ => continue,
                };

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
    }
}
