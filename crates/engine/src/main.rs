mod data_generator;
mod lob;
mod moldudp64_engine;
use bytes::Bytes;
use moldudp64_engine::engine::MoldProducer;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut mold: MoldProducer = MoldProducer::new().await;

    loop {
        sleep(Duration::from_millis(100)).await;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let _ = mold
            .enqueue_message(Bytes::copy_from_slice(&nanos.to_be_bytes()))
            .await;
    }
}
