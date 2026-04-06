use bytes::{BufMut, Bytes, BytesMut};
use mm_core::moldudp64_core::{sessions::SessionTable, types::Event};
use ringbuf::{HeapCons, traits::Consumer};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// A publisher that aggregates market events into MoldUDP64 packets for multicast transmission.
pub struct SequencerPublisher {
    input: HeapCons<Event>,

    session_table: SessionTable,

    multicast_group: SocketAddr,
    multicast_socket: UdpSocket,
    retransmission_socket: UdpSocket,
    packet: BytesMut,

    current_session: Option<[u8; 10]>,
    first_sequence_number: Option<u64>,
    message_count: usize,
    message_history: Vec<Bytes>,
    history_capacity: u64,

    max_packet_size: usize,
    flush_interval: Duration,
    next_flush: Instant,
    running: Arc<AtomicBool>,
}

impl SequencerPublisher {
    /// Initializes a new publisher with a dedicated session ID and prepares the internal transmission buffer.
    pub fn new(
        input: HeapCons<Event>,
        multicast_group: SocketAddr,
        multicast_socket: UdpSocket,
        retransmission_socket: UdpSocket,
        session_id: String,
        running: Arc<AtomicBool>,
    ) -> Self {
        let mut packet = BytesMut::with_capacity(1400);
        packet.resize(20, 0);

        let flush_interval = Duration::from_micros(50);
        let history_capacity = 1_000_000 as u64;

        Self {
            input,
            session_table: SessionTable::new(session_id),
            multicast_group,
            multicast_socket,
            retransmission_socket,
            packet,
            current_session: None,
            first_sequence_number: None,
            message_count: 0,
            message_history: Vec::with_capacity(history_capacity as usize),
            history_capacity,
            max_packet_size: 1400,
            flush_interval,
            next_flush: Instant::now() + flush_interval,
            running,
        }
    }

    /// Finalizes the current packet header and transmits the buffer to the multicast group.
    pub fn flush(&mut self) {
        if self.message_count > 0 {
            self.process_header();

            let _len = self
                .multicast_socket
                .send_to(&self.packet, self.multicast_group)
                .expect("err");

            self.reset();
        }

        self.next_flush = Instant::now() + self.flush_interval;
    }

    /// Runs the main blocking event loop, polling the input queue and flushing based on the configured interval.
    pub fn run(mut self) {
        while self.running.load(Ordering::Relaxed) {
            if Instant::now() >= self.next_flush {
                self.flush();
            }

            while let Some(event) = self.input.try_pop() {
                self.process_event(event);
            }

            std::hint::spin_loop();
        }
    }

    /// Resets the packet buffer and session metadata after a successful transmission.
    #[inline(always)]
    fn reset(&mut self) {
        self.packet.truncate(20);
        self.current_session = None;
        self.first_sequence_number = None;
        self.message_count = 0;
    }

    /// Encodes the session ID, sequence number, and message count into the MoldUDP64 packet header.
    #[inline(always)]
    fn process_header(&mut self) {
        self.packet[0..10].copy_from_slice(&self.current_session.expect("err"));
        self.packet[10..18]
            .copy_from_slice(&(self.first_sequence_number).expect("err").to_be_bytes());
        self.packet[18..20].copy_from_slice(&(self.message_count as u16).to_be_bytes());
    }

    /// Serializes an event into the current packet, triggering a flush if the MTU or session changes.
    #[inline(always)]
    fn process_event(&mut self, event: Bytes) {
        let sequence_number = self.session_table.get_current_sequence_number();
        let session_id = self.session_table.get_current_session();
        let message_length = event.len();
        let total_message_length = 2 + message_length;

        if session_id != self.current_session.expect("err")
            || (self.packet.len() + total_message_length) > self.max_packet_size
        {
            self.flush();

            self.current_session = Some(session_id);
            self.first_sequence_number = Some(sequence_number);
        }

        if self.current_session.is_none() {
            self.first_sequence_number = Some(sequence_number);
            self.current_session = Some(session_id);
        }

        let index = (sequence_number % self.history_capacity) as usize;
        self.message_history[index] = event.clone();

        self.message_count += 1;
        self.packet.put_u16(message_length as u16);
        self.packet.put_slice(&event);

        self.session_table.next_sequence();
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
            UdpSocket::bind("0.0.0.0:8500").expect("err"),
            "MM_L0".to_string(),
            Arc::new(AtomicBool::new(true)),
        )
    }

    #[test]
    fn test_new_initial_state() {
        let sp = make_publisher();

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
