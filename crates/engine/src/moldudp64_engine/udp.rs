use crate::moldudp64_engine::engine::MoldProducer;
use std::io;

impl MoldProducer {
    pub async fn produce(&self, payload: &[u8], addr: std::net::SocketAddr) -> io::Result<usize> {
        let len = self.socket.send_to(payload, addr).await?;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            bytes = len,
            destination = %addr,
            "udp_send"
        );
        Ok(len)
    }
}
