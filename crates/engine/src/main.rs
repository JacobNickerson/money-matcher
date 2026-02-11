use moldudp64_engine::engine::MOLDPRODUCER;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut mold: MOLDPRODUCER = MOLDPRODUCER::new().await;

    for _k in 1..=5 {
        for i in 1..=5 {
            let message: String = OsRng
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();

            println!("Message {:?}: {:?}", i, message);
            mold.enqueue_message(message.as_bytes().to_vec());
        }

        let packet = mold.flush();
        mold.produce(&packet.to_bytes(), "127.0.0.1:8081".parse().unwrap())
            .await?;
    }

    Ok(())
}
