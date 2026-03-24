use bytes::{BufMut, Bytes, BytesMut};
use mm_core::moldudp64_core::sessions::SessionTable;
use mm_core::moldudp64_core::types::Event;
use ringbuf::{HeapCons, traits::Consumer};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub struct SequencerPublisher {
    input: HeapCons<Event>,

    sequence_number: u64,
    session_table: SessionTable,

    multicast_group: SocketAddr,
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
    pub fn new(
        input: HeapCons<Event>,
        multicast_group: SocketAddr,
        socket: UdpSocket,
        session_id: String,
    ) -> Self {
        let mut packet = BytesMut::with_capacity(1400);
        packet.resize(20, 0);

        let flush_interval = Duration::from_micros(5);

        Self {
            input,
            sequence_number: 1,
            session_table: SessionTable::new(session_id),
            multicast_group,
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
            self.process_header();

            let _len = self
                .socket
                .send_to(&self.packet, self.multicast_group)
                .expect("err");

            self.reset();
        }

        self.next_flush = Instant::now() + self.flush_interval;
    }

    pub fn run(mut self) {
        loop {
            if Instant::now() >= self.next_flush {
                self.flush();
            }

            while let Some(event) = self.input.try_pop() {
                self.process_event(event);
            }

            std::hint::spin_loop();
        }
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.packet.truncate(20);
        self.current_session = None;
        self.first_sequence_number = None;
        self.message_count = 0;
    }

    #[inline(always)]
    fn process_header(&mut self) {
        self.packet[0..10].copy_from_slice(&self.current_session.expect("err"));
        self.packet[10..18]
            .copy_from_slice(&(self.first_sequence_number).expect("err").to_be_bytes());
        self.packet[18..20].copy_from_slice(&(self.message_count as u16).to_be_bytes());
    }

    #[inline(always)]
    fn process_event(&mut self, event: Bytes) {
        let sequence_number = self.sequence_number;
        let session_id = self.session_table.get_current_session();

        self.sequence_number += 1;

        if self.current_session.is_none() {
            self.first_sequence_number = Some(sequence_number);
            self.current_session = Some(session_id);
        } else if session_id != self.current_session.expect("err") {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::{HeapRb, traits::Split};

    fn make_publisher() -> SequencerPublisher {
        let (_tx, rx) = HeapRb::<Event>::new(8).split();
        SequencerPublisher::new(
            rx,
            SocketAddr::V4("233.100.10.100:9600".parse().unwrap()),
            UdpSocket::bind("0.0.0.0:0").expect("err"),
            "MM_L0".to_string(),
        )
    }

    #[test]
    fn test_new_initial_state() {
        let sp = make_publisher();

        assert_eq!(sp.sequence_number, 1);
        assert_eq!(sp.packet.len(), 20);
        assert!(sp.current_session.is_none());
        assert!(sp.first_sequence_number.is_none());
        assert_eq!(sp.message_count, 0);
    }

    #[test]
    fn test_process_event_sets_initial_session_and_sequence() {
        let mut sp = make_publisher();

        let e = Bytes::from_static(b"abc");
        sp.process_event(e);

        assert_eq!(sp.sequence_number, 2);
        assert_eq!(sp.message_count, 1);
        assert!(sp.current_session.is_some());
        assert_eq!(sp.first_sequence_number, Some(1));
    }

    #[test]
    fn test_process_event_appends_length_and_payload() {
        let mut sp = make_publisher();

        let e = Bytes::from_static(b"abcd");
        sp.process_event(e);

        let payload = &sp.packet[20..];
        assert_eq!(&payload[0..2], 4u16.to_be_bytes());
        assert_eq!(&payload[2..6], b"abcd");
    }

    #[test]
    fn test_create_header_writes_expected_fields() {
        let mut sp = make_publisher();

        let s = sp.session_table.get_current_session();
        sp.current_session = Some(s);
        sp.first_sequence_number = Some(12);
        sp.message_count = 3;

        sp.process_header();

        assert_eq!(&sp.packet[0..10], &s);
        assert_eq!(&sp.packet[10..18], &12u64.to_be_bytes());
        assert_eq!(&sp.packet[18..20], &3u16.to_be_bytes());
    }

    #[test]
    fn test_reset_clears_packet_state() {
        let mut sp = make_publisher();

        let e = Bytes::from_static(b"abc");
        sp.process_event(e);

        sp.reset();

        assert_eq!(sp.packet.len(), 20);
        assert!(sp.current_session.is_none());
        assert!(sp.first_sequence_number.is_none());
        assert_eq!(sp.message_count, 0);
    }

    #[test]
    fn test_packet_flush_on_max_size_boundary() {
        let mut sp = make_publisher();

        sp.max_packet_size = 24;

        let e1 = Bytes::from_static(b"abcd");
        sp.process_event(e1);

        assert_eq!(sp.message_count, 1);

        let e2 = Bytes::from_static(b"abcd");
        sp.process_event(e2);

        assert_eq!(sp.message_count, 1);
        assert_eq!(sp.first_sequence_number, Some(2));
    }
}
