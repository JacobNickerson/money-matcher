use crate::engine::MOLDPRODUCER;
use std::io;

impl MOLDPRODUCER {
    pub async fn produce(&self, payload: &[u8], addr: std::net::SocketAddr) -> io::Result<usize> {
        let len = self.socket.send_to(payload, addr).await?;
        println!("{:?} bytes sent to {:?}", len, addr);
        Ok(len)
    }
}
