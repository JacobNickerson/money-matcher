use crate::moldudp64_engine::types::*;
use core_affinity2;
use nexus_queue::spsc;
use std::thread;
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;

impl Engine {
    pub fn start() -> Self {
        let (event_tx, event_rx) = spsc::ring_buffer::<Event>(8192);
        let (seq_tx, seq_rx) = spsc::ring_buffer::<SequencedEvent>(8192);

        let sequencer = Sequencer::new(event_rx, seq_tx);
        let publisher = Publisher::new(seq_rx);

        let cores = core_affinity2::get_core_ids().unwrap();

        let seq_core = cores[1];
        let pub_core = cores[2];

        thread::spawn(move || {
            if seq_core.set_affinity().is_ok() {
                sequencer.run();
            }
        });

        thread::spawn(move || {
            if pub_core.set_affinity().is_ok() {
                publisher.run();
            }
        });

        Self { event_tx }
    }

    pub fn push_event(&mut self, event: Event) {
        let _ = self.event_tx.push(event);
    }
}
