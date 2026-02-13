mod udp;

pub struct MOLDCONSUMER {
	socket: tokio::net::UdpSocket,
}