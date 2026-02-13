mod moldudp64_client;
use moldudp64_client::MoldConsumer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mold = MoldConsumer::initialize("127.0.0.1:8081").await?;
    mold.consume().await?;
    Ok(())
}
