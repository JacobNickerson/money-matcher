use crate::moldudp64_client::receiverhandler::ReceiverHandler;
use bytes::Bytes;
use netlib::moldudp64_core::types::ItchEvent;
use nexus_queue::spsc;
use std::thread;
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;
pub struct Client {
    handler_rx: spsc::Consumer<ItchEvent>,
}

impl Client {
    pub fn start() -> Self {
        #[cfg(feature = "tracing")]
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();

        #[cfg(feature = "tracing")]
        tracing::subscriber::set_global_default(subscriber).expect("tracing init failed");

        let (handler_tx, handler_rx) = spsc::ring_buffer::<ItchEvent>(8192);

        let receiver_handler = ReceiverHandler::new(handler_tx);

        thread::spawn(move || {
            receiver_handler.run();
        });

        Self { handler_rx }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use std::time::{SystemTime, UNIX_EPOCH};

    use netlib::moldudp64_core::types::TestBenchmark;

    use super::*;

    #[test]
    fn benchmark_mold_consumer_enqueue() {
        let mut client = Client::start();
        let mut count = 0u64;
        let mut start_instant: Option<Instant> = None;

        loop {
            if let Some(event) = client.handler_rx.pop() {
                match event {
                    ItchEvent::TestBenchmark(s) => {
                        if start_instant.is_none() {
                            start_instant = Some(Instant::now());
                            tracing::debug!("start_received");
                        } else if let Some(start) = start_instant {
                            let elapsed = start.elapsed().as_nanos() as u64;

                            tracing::debug!(
                                total_time_ns = elapsed,
                                average_latency_ns = elapsed / count,
                                count = count,
                                "benchmark_results"
                            );

                            break;
                        }
                    }

                    ItchEvent::AddOrder(s) => {
                        count += 1;
                    }

                    _ => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("unknown_message_type");
                    }
                }
            } else {
                std::hint::spin_loop();
            }
        }
    }
}
