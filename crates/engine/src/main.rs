mod moldudp64_engine;
use bytes::Bytes;
use moldudp64_engine::engine::MoldProducer;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut mold: MoldProducer = MoldProducer::new().await;
    let mut rng = rand::thread_rng();

    loop {
        sleep(Duration::from_millis(100)).await;

        let mut message: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(rng.gen_range(32..=512))
            .map(char::from)
            .collect();

        message += " COMPLETE";

        let _ = mold
            .enqueue_message(Bytes::copy_from_slice(message.as_bytes()))
            .await;
    }
}
