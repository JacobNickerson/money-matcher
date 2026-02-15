use crate::moldudp64_client::MoldConsumer;
use bytes::BytesMut;
use netlib::moldudp64_core::types::Packet;
use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::net::UdpSocket;
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;

impl MoldConsumer {
    pub async fn initialize(bind_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        #[cfg(feature = "tracing")]
        tracing::info!(bind_addr = bind_addr, "mold_consumer_initialized");

        #[cfg(feature = "tracing")]
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish();

        #[cfg(feature = "tracing")]
        tracing::subscriber::set_global_default(subscriber).expect("tracing init failed");

        Ok(MoldConsumer { socket })
    }

    pub async fn consume(&self) -> io::Result<()> {
        let mut buf = BytesMut::with_capacity(2048);

        loop {
            buf.resize(2048, 0);

            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            #[cfg(feature = "tracing")]
            tracing::trace!(
                bytes = len,
                source = %addr,
                "udp_receive"
            );

            let bytes = buf.split_to(len).freeze();
            let packet = Packet::from_bytes(bytes).expect("invalid packet");
            let header = packet.header;
            let message_count = u16::from_be_bytes(header.message_count) as usize;

            #[cfg(feature = "tracing")]
            tracing::debug!(
                source = %addr,
                session = %std::str::from_utf8(&header.session_id).unwrap(),
                sequence = u64::from_be_bytes(header.sequence_number),
                count = message_count,
                "packet_received"
            );

            let message_blocks = packet.message_blocks;
            let mut k = 1;

            for msg in message_blocks {
                let message_type = msg.message_data[0];
                let message_char = message_type as char;

                #[cfg(feature = "tracing")]
                tracing::trace!(
                    index = k,
                    message_type = %message_char,
                    length = u16::from_be_bytes(msg.message_length),
                    "message_block"
                );

                match message_type {
                    b'b' => {
                        let nanos =
                            u128::from_be_bytes(msg.message_data[1..17].try_into().unwrap());

                        let time = UNIX_EPOCH + Duration::from_nanos(nanos as u64);
                        let now = SystemTime::now();
                        let elapsed = now.duration_since(time).unwrap();

                        #[cfg(feature = "tracing")]
                        tracing::trace!(
                            index = k,
                            latency =
                                format!("{}.{:09}", elapsed.as_secs(), elapsed.subsec_nanos()),
                            "benchmark_latency"
                        );
                    }
                    _ => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(
                            index = k,
                            message_type = %message_char,
                            "unknown_message_type"
                        );
                    }
                }

                k += 1;
            }

            buf.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn benchmark_mold_consumer_enqueue() -> std::io::Result<()> {
        let mold = MoldConsumer::initialize("127.0.0.1:8081").await?;
        mold.consume().await?;
        Ok(())
    }
}
