mod udp;

pub struct MoldConsumer {
    socket: tokio::net::UdpSocket,
}
