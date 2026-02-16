use bytes::{BufMut, BytesMut};
use netlib::moldudp64_core::sessions::SessionTable;
use netlib::moldudp64_core::types::Event;
use nexus_queue::spsc;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

pub struct SequencerPublisher {
    input: spsc::Consumer<Event>,

    sequence_number: u64,
    session_table: SessionTable,

    socket: UdpSocket,
    packet: BytesMut,

    current_session: Option<[u8; 10]>,
    first_sequence_number: Option<u64>,
    message_count: usize,

    max_packet_size: usize,
    flush_interval: Duration,
    next_flush: Instant,
}

impl SequencerPublisher {
    pub fn new(input: spsc::Consumer<Event>) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
        let mut packet = BytesMut::with_capacity(1400);
        packet.resize(20, 0);

        let flush_interval = Duration::from_micros(5);

        Self {
            input,
            sequence_number: 1,
            session_table: SessionTable::new(),
            socket,
            packet,
            current_session: None,
            first_sequence_number: None,
            message_count: 0,
            max_packet_size: 1400,
            flush_interval,
            next_flush: Instant::now() + flush_interval,
        }
    }

    pub fn flush(&mut self) {
        if self.message_count > 0 {
            self.packet[0..10].copy_from_slice(&self.current_session.unwrap());
            self.packet[10..18]
                .copy_from_slice(&(self.first_sequence_number).unwrap().to_be_bytes());
            self.packet[18..20].copy_from_slice(&(self.message_count as u16).to_be_bytes());

            let addr = "127.0.0.1:8081";
            let _len = self.socket.send_to(&self.packet, addr).unwrap();

            self.packet.truncate(20);

            self.current_session = None;
            self.first_sequence_number = None;
            self.message_count = 0;
        }

        self.next_flush = Instant::now() + self.flush_interval;
    }

    pub fn run(mut self) {
        loop {
            if Instant::now() >= self.next_flush {
                self.flush();
            }

            if let Some(event) = self.input.pop() {
                let sequence_number = self.sequence_number;
                let session_id = self.session_table.get_current_session();

                self.sequence_number += 1;

                if self.current_session.is_none() {
                    self.first_sequence_number = Some(sequence_number);
                    self.current_session = Some(session_id);
                } else if session_id != self.current_session.unwrap() {
                    self.flush();

                    self.current_session = Some(session_id);
                    self.first_sequence_number = Some(sequence_number);
                }

                let message_length = event.len();
                let total_message_length = 2 + message_length;

                if (self.packet.len() + total_message_length) > self.max_packet_size {
                    self.flush();

                    self.current_session = Some(session_id);
                    self.first_sequence_number = Some(sequence_number);
                }

                self.message_count += 1;

                self.packet.put_u16(message_length as u16);
                self.packet.put_slice(&event);
            }

            std::hint::spin_loop();
        }
    }
}
