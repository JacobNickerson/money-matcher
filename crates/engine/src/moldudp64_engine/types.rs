use bytes::{Bytes, BytesMut};
use netlib::moldudp64_core::sessions::SessionTable;
use nexus_queue::spsc;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

pub struct Event {
    pub payload: Bytes,
}
pub struct SequencedEvent {
    pub payload: Bytes,
    pub sequence_number: u64,
    pub session_id: [u8; 10],
}

pub struct Sequencer {
    pub input: spsc::Consumer<Event>,
    pub output: spsc::Producer<SequencedEvent>,
    pub sequence_number: u64,
    pub session_table: SessionTable,
}

pub struct Publisher {
    pub current_session: Option<[u8; 10]>,
    pub first_sequence_number: Option<u64>,
    pub flush_interval: Duration,
    pub input: spsc::Consumer<SequencedEvent>,
    pub max_packet_size: usize,
    pub message_count: usize,
    pub next_flush: Instant,
    pub packet_size: usize,
    pub packet: BytesMut,
    pub socket: UdpSocket,
}

pub struct Engine {
    pub event_tx: spsc::Producer<Event>,
}
