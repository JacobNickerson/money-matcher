use mm_core::lob_core::market_events::{MarketEventType, SingleEventFeed};
use mm_core::lob_core::{
    market_orders::Order,
    market_events::MarketEvent,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ringbuf::{HeapRb, traits::*};
use std::thread;
use std::time::Instant;
use std::fs::File;
use std::io::{self, Write};

use crate::data_generator::event_source::PoissonSource;
use crate::data_generator::order_generators::GaussianOrderGenerator;
use crate::data_generator::rate_controllers::ConstantPoissonRate;
use crate::data_generator::type_selectors::UniformTypeSelector;
use crate::simulator::simulator::Simulator;

mod data_generator;
mod fix;
mod lob;
mod simulator;

fn main() {
    let (mut user_order_prod, mut user_order_cons) = HeapRb::<Order>::new(1 << 24).split();
    let (mut market_event_prod, mut market_event_cons) =
        HeapRb::<MarketEvent>::new(1 << 24).split();
    let mut sim = Simulator::new(
        PoissonSource::new(
            ConstantPoissonRate::new(100_000.0),
            UniformTypeSelector::new(0.5, 0.4,0.3,0.2,0.1),
            GaussianOrderGenerator::new(1000.0, 500.0),
            ChaCha8Rng::seed_from_u64(0),
        ),
        SingleEventFeed::new(market_event_prod),
        user_order_cons,
        ChaCha8Rng::seed_from_u64(67),
    );
    let mut trade_file = File::create("trades.txt").unwrap();
    let mut order_file = File::create("orders.txt").unwrap();
    let event_logger_thread = thread::spawn(move || {
        loop {
            if let Some(order) = market_event_cons.try_pop() {
                match order.kind {
                    MarketEventType::L3(order) => {
                        writeln!(order_file,"{:?}",order).unwrap();
                    },
                    MarketEventType::Trade(trade) => {
                        writeln!(trade_file,"{:?}",trade).unwrap();
                    },
                    _ => {},
                }
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
    let _ = event_logger_thread.join();
}
