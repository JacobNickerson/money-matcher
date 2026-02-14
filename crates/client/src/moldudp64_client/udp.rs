use crate::moldudp64_client::MoldConsumer;
use bytes::BytesMut;
use netlib::moldudp64_core::types::Packet;
use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::net::UdpSocket;

impl MoldConsumer {
    pub async fn initialize(bind_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("Initialized MoldConsumer on {}", bind_addr);
        Ok(MoldConsumer { socket })
    }

    pub async fn consume(&self) -> io::Result<()> {
        let mut buf = BytesMut::with_capacity(2048);

        loop {
            buf.resize(2048, 0);

            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            println!();
            println!("{:?} bytes received from {:?}", len, addr);

            let bytes = buf.split_to(len).freeze();
            let packet = Packet::from_bytes(bytes).expect("invalid packet");

            let header = packet.header;
            println!(
                "Header Session ID: {:?} {:?}",
                std::str::from_utf8(&header.session_id).unwrap(),
                header.session_id
            );
            println!(
                "Header Sequence Number: {:?} {:?}",
                u64::from_be_bytes(header.sequence_number) as usize,
                header.sequence_number
            );
            let message_count = u16::from_be_bytes(header.message_count) as usize;
            println!(
                "Header Message Count: {:?} {:?}",
                message_count, header.message_count
            );

            let message_blocks = packet.message_blocks;
            let mut k = 1;

            for msg in message_blocks {
                let message_type = msg.message_data[0];

                println!(
                    "{:?} Message {:?}: {:?}",
                    message_type as char, k, msg.message_length
                );

                match message_type {
                    b'b' => {
                        let nanos =
                            u128::from_be_bytes(msg.message_data[1..17].try_into().unwrap());

                        let time = UNIX_EPOCH + Duration::from_nanos(nanos as u64);
                        let now = SystemTime::now();
                        let elapsed = now.duration_since(time).unwrap();

                        println!(
                            "Elapsed: {}.{:09}s",
                            elapsed.as_secs(),
                            elapsed.subsec_nanos()
                        );
                    }
                    _ => {
                        println!("Message {:?} UNKNOWN Type: {:?}", k, message_type as char);
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
