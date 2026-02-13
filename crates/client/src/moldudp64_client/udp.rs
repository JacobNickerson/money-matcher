use crate::MoldConsumer;
use bytes::BytesMut;
use netlib::moldudp64_core::types::Packet;
use std::io;
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
                let message: String = std::str::from_utf8(&msg.message_data).unwrap().to_string();
                println!("Message {:?}: {:?}", k, message);
                k += 1;
            }

            buf.clear();
        }
    }
}
