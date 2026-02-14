#[cfg(test)]
mod tests {
    use crate::moldudp64_client::*;

    #[tokio::test]
    async fn benchmark_mold_consumer_enqueue() -> std::io::Result<()> {
        let mold = MoldConsumer::initialize("127.0.0.1:8081").await?;
        mold.consume().await?;
        Ok(())
    }
}
