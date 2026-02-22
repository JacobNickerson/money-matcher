use crate::moldudp64_engine::sequencerpublisher::SequencerPublisher;
use bytes::Bytes;
use netlib::{itch_core::messages::HasTrackingNumber, moldudp64_core::types::Event};
use nexus_queue::{Full, spsc};
use std::{os::raw, thread};
use zerocopy::{Immutable, IntoBytes};

pub struct MoldEngine {
    event_tx: spsc::Producer<Event>,
    current_tracking_number: u16,
}

impl MoldEngine {
    pub fn start() -> Self {
        let (event_tx, event_rx) = spsc::ring_buffer::<Event>(8192);
        let sequencer_publisher = SequencerPublisher::new(event_rx);

        thread::spawn(move || {
            sequencer_publisher.run();
        });

        Self {
            event_tx,
            current_tracking_number: 1,
        }
    }

    pub fn push_event<T>(&mut self, mut event: T)
    where
        T: IntoBytes + Immutable + HasTrackingNumber,
    {
        loop {
            event.set_tracking_number(self.current_tracking_number);
            self.current_tracking_number += 1;

            let mut bytes = Bytes::copy_from_slice(event.as_bytes());
            match self.event_tx.push(bytes) {
                Ok(_) => break,
                Err(Full(e)) => {
                    bytes = e;
                    std::hint::spin_loop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use netlib::itch_core::types::{AddOrder, TestBenchmark};
    use std::{
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use zerocopy::IntoBytes;

    #[test]
    fn benchmark_mold_producer_enqueue() -> std::io::Result<()> {
        let mut engine = MoldEngine::start();

        let bench = TestBenchmark::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );

        engine.push_event(bench);
        let mut next = Instant::now();

        let mut stock = [b' '; 8];
        let st = "STOCK";
        stock[..st.len()].copy_from_slice(st.as_bytes());
        for _i in 0..1000000 {
            let order = AddOrder::new(1, 123456, 1000, b'A', 100, stock, 1);

            engine.push_event(order);

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

        engine.push_event(bench);
        thread::sleep(Duration::from_secs(1));

        Ok(())
    }
}
