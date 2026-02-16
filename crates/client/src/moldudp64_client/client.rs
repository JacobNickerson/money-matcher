use crate::moldudp64_client::receiverhandler::ReceiverHandler;
use netlib::itch_core::types::ItchEvent;
use nexus_queue::spsc;
use std::thread;
pub struct Client {
    handler_rx: spsc::Consumer<ItchEvent>,
}

impl Client {
    pub fn start() -> Self {
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
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_mold_consumer_enqueue() {
        let mut client = Client::start();
        let mut count = 0u64;
        let mut start_instant: Option<Instant> = None;

        loop {
            if let Some(event) = client.handler_rx.pop() {
                match event {
                    ItchEvent::TestBenchmark(_s) => {
                        if start_instant.is_none() {
                            start_instant = Some(Instant::now());
                            println!("Start benchmark");
                        } else if let Some(start) = start_instant {
                            let elapsed = start.elapsed().as_nanos() as u64;

                            println!(
                                "	benchmark_results total_time_ns={} average_latency_ns={} count={}",
                                elapsed,
                                elapsed / count,
                                count
                            );

                            break;
                        }
                    }

                    ItchEvent::AddOrder(_s) => {
                        count += 1;
                    }

                    _ => {}
                }
            } else {
                std::hint::spin_loop();
            }
        }
    }
}
