use bytes::BytesMut;
use netlib::moldudp64_core::sessions::SessionTable;
use netlib::moldudp64_core::types::*;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
pub struct MoldProducer {
    pub socket: UdpSocket,
    session_table: SessionTable,
    pub(crate) packet: BytesMut,
    last_flush: Instant,
    flush_interval: Duration,
    message_count: usize,
    max_messages: usize,
    packet_size: usize,
    max_packet_size: usize,
}

impl MoldProducer {
    pub async fn new() -> Self {
        let mut packet = BytesMut::with_capacity(1400);
        packet.resize(20, 0);

        MoldProducer {
            socket: UdpSocket::bind("0.0.0.0:9000").await.unwrap(),
            session_table: SessionTable::new(),
            message_count: 0,
            max_messages: 65535,
            packet,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(500),
            packet_size: 20,
            max_packet_size: 1400,
        }
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        let session_id: [u8; 10] = self.session_table.get_current_session();
        let sequence_number = self.session_table.next_sequence(session_id);

        self.packet[0..10].copy_from_slice(&session_id);
        self.packet[10..18].copy_from_slice(&sequence_number);
        self.packet[18..20].copy_from_slice(&(self.message_count as u16).to_be_bytes());

        self.produce(&self.packet, "127.0.0.1:8081".parse().unwrap())
            .await?;

        self.packet.truncate(20);

        self.packet_size = 20;
        self.message_count = 0;
        self.last_flush = Instant::now();

        Ok(())
    }

    pub async fn enqueue_message(&mut self, message: MessageData) -> std::io::Result<()> {
        let message_length = message.len();
        let total_message_length = 2 + message_length;
        if (self.packet_size + total_message_length) > self.max_packet_size {
            println!();
            println!("Flushing messages before reaching 1400 bytes");
            println!("Current Bytes: {:?}", self.packet_size);
            println!("Message Bytes: {:?}", total_message_length);
            println!("Total Bytes: {:?}", self.packet_size + total_message_length);
            self.flush().await?;
        }

        self.packet_size += total_message_length;
        self.message_count += 1;

        self.packet
            .extend_from_slice(&(message_length as u16).to_be_bytes());
        self.packet.extend_from_slice(&message);

        if self.packet.len() >= self.max_messages {
            println!();
            println!("Flushing messages due to packet reaching capacity");
            self.flush().await?;
        }

        if self.last_flush.elapsed() >= self.flush_interval {
            println!();
            println!("Flushing messages due to timer");
            self.flush().await?;
        }

        Ok(())
    }
}
