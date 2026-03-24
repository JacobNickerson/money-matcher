use mm_core::lob_core::market_events::{EventSink, MarketEventType, SingleEventFeed};
use mm_core::lob_core::{market_events::MarketEvent, market_orders::Order};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ringbuf::{HeapRb, traits::*};
use std::fs::File;
use std::io::{self, Write};
use std::thread;
use std::time::Instant;

use crate::data_generator::event_source::PoissonSource;
use crate::data_generator::order_generators::GaussianOrderGenerator;
use crate::data_generator::rate_controllers::ConstantPoissonRate;
use crate::data_generator::type_selectors::UniformTypeSelector;
use crate::moldudp64::engine::MoldEngine;
use crate::simulator::simulator::Simulator;

mod data_generator;
mod fix;
mod lob;
mod moldudp64;
mod simulator;

fn main() {
    let (mut user_order_prod, mut user_order_cons) = HeapRb::<Order>::new(1 << 24).split();
    let (mut market_event_prod, mut market_event_cons) =
        HeapRb::<MarketEvent>::new(1 << 24).split();
    let mut sim = Simulator::new(
        PoissonSource::new(
            ConstantPoissonRate::new(100_000.0),
            UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
            GaussianOrderGenerator::new(1000.0, 500.0),
            ChaCha8Rng::seed_from_u64(0),
        ),
        SingleEventFeed::new(market_event_prod),
        user_order_cons,
        ChaCha8Rng::seed_from_u64(67),
    );
    let mut mold_engine = MoldEngine::start();
    let event_broadcast_thread = thread::spawn(move || {
        loop {
            if let Some(order) = market_event_cons.try_pop() {
                mold_engine.push(order);
            }
        }
    });
    let time = Instant::now();
    for _ in 0..1_000_000 {
        #[allow(dead_code)]
        sim.step();
    }
    let elapsed = time.elapsed();
    println!(
        "Sim time: {}s ({}ns)",
        sim.time() as f64 / 1_000_000_000.0,
        sim.time()
    );
    println!(
        "Real time: {}s ({}ns)",
        elapsed.as_nanos() as f64 / 1_000_000_000.0,
        elapsed.as_nanos()
    );
    let _ = event_broadcast_thread.join();
}
