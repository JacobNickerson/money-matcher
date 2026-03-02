use crate::moldudp64_engine::sequencerpublisher::SequencerPublisher;
use bytes::Bytes;
use netlib::{itch_core::messages::ItchMessage, moldudp64_core::types::Event};
use nexus_queue::{Full, spsc};
use std::{net::UdpSocket, thread};
use zerocopy::{Immutable, IntoBytes};

pub struct MoldEngine {
    event_tx: spsc::Producer<Event>,
    current_tracking_number: u16,
}

impl MoldEngine {
    pub fn start() -> Self {
        let (event_tx, event_rx) = spsc::ring_buffer::<Event>(8192);
        let sequencer_publisher =
            SequencerPublisher::new(event_rx, UdpSocket::bind("0.0.0.0:9000").expect("err"));

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
        T: IntoBytes + Immutable + ItchMessage,
    {
        loop {
            event.set_tracking_number(self.current_tracking_number);
            self.current_tracking_number = self.current_tracking_number.wrapping_add(1);

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
    use netlib::itch_core::messages::{add_order::AddOrder, test_benchmark::TestBenchmark};
    use std::{
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use zerocopy::IntoBytes;

    #[test]
    #[ignore]
    fn benchmark_mold_producer_enqueue() -> std::io::Result<()> {
        let mut engine = MoldEngine::start();

        let bench1 = TestBenchmark::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("err")
                .as_nanos() as u64,
        );

        engine.push_event(bench1);

        let mut next = Instant::now();

        let mut stock = [b' '; 8];
        let s1 = "STOCK";
        stock[..s1.len()].copy_from_slice(s1.as_bytes());

        for _i in 0..1000000 {
            let order = AddOrder::new(1, 123456, 1000, b'A', 100, stock, 1.into());

            engine.push_event(order);

            next += Duration::from_nanos(1000);
            while Instant::now() < next {
                std::hint::spin_loop();
            }
        }

        let bench2 = TestBenchmark::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("err")
                .as_nanos() as u64,
        );

        engine.push_event(bench2);

        thread::sleep(Duration::from_secs(1));

        Ok(())
    }

    #[test]
    fn test_push_event_increments_tracking_number() {
        let (tx, _rx) = spsc::ring_buffer::<Event>(8);

        let mut engine = MoldEngine {
            event_tx: tx,
            current_tracking_number: 1,
        };

        let e = TestBenchmark::new(0_u64);

        engine.push_event(e);

        assert_eq!(engine.current_tracking_number, 2);
    }
}
