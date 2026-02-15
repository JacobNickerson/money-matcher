use crate::moldudp64_engine::types::*;
use bytes::{BufMut, BytesMut};
use nexus_queue::spsc;
use std::net::UdpSocket;
use std::time::{Duration, Instant};
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;

impl Publisher {
    pub fn new(input: spsc::Consumer<SequencedEvent>) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
        let mut packet = BytesMut::with_capacity(1400);
        packet.resize(20, 0);

        #[cfg(feature = "tracing")]
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish();

        #[cfg(feature = "tracing")]
        tracing::subscriber::set_global_default(subscriber).expect("tracing init failed");

        let flush_interval = Duration::from_millis(500);

        Self {
            current_session: None,
            first_sequence_number: None,
            flush_interval,
            input,
            max_packet_size: 1400,
            message_count: 0,
            next_flush: Instant::now() + flush_interval,
            packet_size: 20,
            packet,
            socket,
        }
    }

    pub fn flush(&mut self) {
        if self.message_count > 0 {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                sequence_number = self.first_sequence_number.unwrap(),
                message_count = self.message_count,
                packet_bytes = self.packet_size,
                "packet_flushed"
            );

            self.packet[0..10].copy_from_slice(&self.current_session.unwrap());
            self.packet[10..18]
                .copy_from_slice(&(self.first_sequence_number).unwrap().to_be_bytes());
            self.packet[18..20].copy_from_slice(&(self.message_count as u16).to_be_bytes());

            let addr = "127.0.0.1:8081";
            let len = self.socket.send_to(&self.packet, addr).unwrap();
            #[cfg(feature = "tracing")]
            tracing::debug!(bytes = len, destination = addr, "udp_send");

            self.packet.truncate(20);

            self.current_session = None;
            self.first_sequence_number = None;
            self.message_count = 0;
            self.packet_size = 20;
        }

        self.next_flush += self.flush_interval;
    }

    pub fn run(mut self) {
        loop {
            if let Some(sequenced_event) = self.input.pop() {
                if self.current_session.is_none() {
                    self.first_sequence_number = Some(sequenced_event.sequence_number);
                    self.current_session = Some(sequenced_event.session_id);
                } else if sequenced_event.session_id != self.current_session.unwrap() {
                    self.flush();

                    self.current_session = Some(sequenced_event.session_id);
                    self.first_sequence_number = Some(sequenced_event.sequence_number);
                }

                let message_length = sequenced_event.payload.len();
                let total_message_length = 2 + message_length;
                if (self.packet_size + total_message_length) > self.max_packet_size {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        reason = "capacity",
                        current = self.packet_size,
                        incoming = total_message_length,
                        total = self.packet_size + total_message_length,
                        max = self.max_packet_size,
                        "flush"
                    );

                    self.flush();

                    self.current_session = Some(sequenced_event.session_id);
                    self.first_sequence_number = Some(sequenced_event.sequence_number);
                }

                self.message_count += 1;
                self.packet_size += total_message_length;

                #[cfg(feature = "tracing")]
                tracing::debug!(
                    message_index = self.message_count,
                    message_bytes = total_message_length,
                    packet_bytes = self.packet_size,
                    "enqueue"
                );

                self.packet.put_u16(message_length as u16);
                self.packet.put_slice(&sequenced_event.payload);
            } else {
                if Instant::now() >= self.next_flush {
                    self.flush();
                }

                std::thread::yield_now();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn benchmark_mold_producer_enqueue() -> std::io::Result<()> {
        let mut engine = Engine::start();

        for _ in 0..100 {
            thread::sleep(Duration::from_millis(100));

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let mut msg = BytesMut::with_capacity(17);
            msg.put_u8(b'b');
            msg.extend_from_slice(&nanos.to_be_bytes());
            let event = Event {
                payload: msg.freeze(),
            };

            engine.push_event(event);
        }

        Ok(())
    }
}
