use netlib::moldudp64_core::sessions::SessionTable;
use netlib::moldudp64_core::types::*;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
pub struct MoldProducer {
    pub socket: UdpSocket,
    session_table: SessionTable,
    pub(crate) message_queue: MessageBlocks,
    last_flush: Instant,
    flush_interval: Duration,
    max_messages: usize,
    packet_size: usize,
    max_packet_size: usize,
}

impl MoldProducer {
    pub async fn new() -> Self {
        MoldProducer {
            socket: UdpSocket::bind("0.0.0.0:9000").await.unwrap(),
            session_table: SessionTable::new(),
            max_messages: 65535,
            message_queue: Vec::with_capacity(65535),
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(500),
            packet_size: 20,
            max_packet_size: 1400,
        }
    }

    pub fn make_packet(&mut self, message_blocks: MessageBlocks) -> Packet {
        let session_id = self.session_table.get_current_session();
        let sequence_number = self.session_table.next_sequence(session_id);
        let message_count = (message_blocks.len() as u16).to_be_bytes();
        let header = Header {
            session_id,
            sequence_number,
            message_count,
        };

        Packet {
            header,
            message_blocks,
        }
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        let messages = std::mem::take(&mut self.message_queue);
        let packet = self.make_packet(messages);

        self.produce(&packet.to_bytes(), "127.0.0.1:8081".parse().unwrap())
            .await?;

        self.packet_size = 20;
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
        self.message_queue.push(MessageBlock {
            message_length: (message_length as u16).to_be_bytes(),
            message_data: message,
        });

        if self.message_queue.len() >= self.max_messages {
            println!();
            println!("Flushing messages due to message_queue reaching capacity");
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
