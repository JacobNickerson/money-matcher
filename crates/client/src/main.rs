mod moldudp64_client;
use moldudp64_client::MOLDCONSUMER;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mold = MOLDCONSUMER::initialize("127.0.0.1:8081").await?;
    mold.consume().await?;
    Ok(())
}
