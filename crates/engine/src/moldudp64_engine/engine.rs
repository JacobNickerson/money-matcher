use crate::moldudp64_engine::sequencerpublisher::SequencerPublisher;
use netlib::moldudp64_core::types::{Event, SequencedEvent};
use nexus_queue::{Full, spsc};
use std::thread;
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;
pub struct Engine {
    event_tx: spsc::Producer<Event>,
}

impl Engine {
    pub fn start() -> Self {
        #[cfg(feature = "tracing")]
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();

        #[cfg(feature = "tracing")]
        tracing::subscriber::set_global_default(subscriber).expect("tracing init failed");

        let (event_tx, event_rx) = spsc::ring_buffer::<Event>(8192);

        let sequencer_publisher = SequencerPublisher::new(event_rx);

        thread::spawn(move || {
            sequencer_publisher.run();
        });

        Self { event_tx }
    }

    pub fn push_event(&mut self, mut event: Event) {
        loop {
            match self.event_tx.push(event) {
                Ok(_) => break,
                Err(Full(e)) => {
                    event = e;
                    std::hint::spin_loop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use netlib::moldudp64_core::types::{AddOrder, TestBenchmark};
    use std::{
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use zerocopy::IntoBytes;

    #[test]
    fn benchmark_mold_producer_enqueue() -> std::io::Result<()> {
        let mut engine = Engine::start();

        let bench = TestBenchmark::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );

        engine.push_event(Bytes::copy_from_slice(bench.as_bytes()));
        let mut next = Instant::now();

        for i in 0..5000000 {
            let mut stock = [b' '; 8];
            stock[..4].copy_from_slice(b"AAAA");
            let order = AddOrder::new(1, 1, 123456, 1000, b'A', 100, stock, 1);

            engine.push_event(Bytes::copy_from_slice(order.as_bytes()));

            next += Duration::from_nanos(1000);
            while Instant::now() < next {
                std::hint::spin_loop();
            }
        }

        let bench = TestBenchmark::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );

        engine.push_event(Bytes::copy_from_slice(bench.as_bytes()));
        thread::sleep(Duration::from_secs(10));

        Ok(())
    }
}
