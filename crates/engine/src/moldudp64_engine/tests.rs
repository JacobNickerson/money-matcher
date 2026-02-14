#[cfg(test)]
mod tests {
    use crate::moldudp64_engine;
    use bytes::{BufMut, BytesMut};
    use moldudp64_engine::engine::MoldProducer;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn benchmark_mold_producer_enqueue() -> std::io::Result<()> {
        let mut mold: MoldProducer = MoldProducer::new().await;

        for _ in 0..100 {
            sleep(Duration::from_millis(100)).await;

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let mut msg = BytesMut::with_capacity(17);
            msg.put_u8(b'Z');
            msg.extend_from_slice(&nanos.to_be_bytes());

            mold.enqueue_message(msg.freeze()).await?;

            let mut msg = BytesMut::with_capacity(17);
            msg.put_u8(b'B');
            msg.extend_from_slice(&nanos.to_be_bytes());

            mold.enqueue_message(msg.freeze()).await?;
        }

        Ok(())
    }
}
