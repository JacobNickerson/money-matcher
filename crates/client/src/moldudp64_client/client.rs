use crate::moldudp64_client::receiverhandler::ReceiverHandler;
use netlib::itch_core::types::ItchEvent;
use nexus_queue::spsc;
use std::{net::UdpSocket, thread};
pub struct MoldClient {
    handler_rx: spsc::Consumer<ItchEvent>,
}

impl MoldClient {
    pub fn start() -> Self {
        let (handler_tx, handler_rx) = spsc::ring_buffer::<ItchEvent>(8192);
        let receiver_handler =
            ReceiverHandler::new(handler_tx, UdpSocket::bind("127.0.0.1:8081").expect("err"));

        thread::spawn(move || {
            receiver_handler.run();
        });

        Self { handler_rx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    #[ignore]
    fn benchmark_mold_consumer_enqueue() {
        let mut mold_client = MoldClient::start();
        let mut count = 0u64;
        let mut start_instant: Option<Instant> = None;

        loop {
            if let Some(event) = mold_client.handler_rx.pop() {
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

    #[test]
    #[ignore]
    fn receive_orders() {
        let mut mold_client = MoldClient::start();

        loop {
            if let Some(event) = mold_client.handler_rx.pop() {
                match event {
                    ItchEvent::AddOrder(_s) => {
                        _s.print();
                    }

                    _ => {
                        println!("received something..")
                    }
                }
            } else {
                std::hint::spin_loop();
            }
        }
    }
}
