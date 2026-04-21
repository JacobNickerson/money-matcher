use clap::Parser;
use mm_core::fix_core::messages::execution_report::ExecutionReport;
use mm_core::fix_core::messages::{FIXEvent, FIXPayload, ReportMessage};
use mm_core::lob_core::market_events::{ClientEvent, SingleEventFeed};
use mm_core::lob_core::{market_events::MarketEvent, market_orders::Order};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ringbuf::{HeapRb, traits::*};
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Instant;

use crate::data_generator::event_source::{EventSource, FileReplaySource, PoissonSource, SourceEnum};
use crate::data_generator::order_generators::GaussianOrderGenerator;
use crate::data_generator::rate_controllers::ConstantPoissonRate;
use crate::data_generator::type_selectors::UniformTypeSelector;
use crate::fix::engine::FixEngine;
use crate::moldudp64::engine::MoldEngine;
use crate::simulator::Simulator;
use crate::simulator::latency_config::{JitterCfg, LatencyConfig, SimJitter};

use engine::{positive_float_parser, prob_parser};

mod data_generator;
mod fix;
mod moldudp64;
mod simulator;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Synthetic order rate in orders/second
    #[arg(long, default_value_t = 100_000.0)]
    order_rate: f64,

    /// Number of orders to generate before terminating, if unused the simulation runs indefinitely
    #[arg(long)]
    count: Option<u64>,

    /// Proportion of synthetic orders that are bids vs asks, value must be between 0-1
    #[arg(long, default_value_t = 0.5, value_parser = prob_parser)]
    bid_rate: f64,

    /// Proportion of synthetic orders that are new limits, value must be between 0-1 and must sum to 1 with the other event types
    #[arg(long, default_value_t = 0.5, value_parser = prob_parser)]
    new_limit_rate: f64,

    /// Proportion of synthetic orders that are cancels, value must be between 0-1 and must sum to 1 with the other event types
    #[arg(long, default_value_t = 0.4, value_parser = prob_parser)]
    cancel_rate: f64,

    /// Proportion of synthetic orders that are market orders, value must be between 0-1 and must sum to 1 with the other event types
    #[arg(long, default_value_t = 0.05, value_parser = prob_parser)]
    market_rate: f64,

    /// Proportion of synthetic orders that are updates, value must be between 0-1 and must sum to 1 with the other event types
    #[arg(long, default_value_t = 0.05, value_parser = prob_parser)]
    update_rate: f64,

    /// Average order price in cents, must be a positive, non-zero value
    #[arg(long, default_value_t = 1000.0, value_parser = positive_float_parser)]
    avg_price: f64,

    /// Standard deviation of order price in cents, must be a positive, non-zero value
    #[arg(long, default_value_t = 50.0, value_parser = positive_float_parser)]
    price_dev: f64,

    /// RNG seed for randomly sampled values, if unspecified a random one is picked
    #[arg(long)]
    seed: Option<u64>,

    /// Simulated latency in nanoseconds
    #[arg(long, default_value_t = 0)]
    sim_latency: u64,

    /// Configure simulated jitter
    #[command(subcommand)]
    sim_jitter: Option<JitterCfg>,

    /// Attempt to run the simulation in real-time by attempting to keep sim time and real time synchronized
    #[arg(long)]
    real_time: bool,

    /// Log errors and initialization steps to stdout
    #[arg(long, default_value_t = false)]
    logging: bool,

    /// Records runtime and events processed and outputs to stdout after simulator finishes generating orders
    ///
    /// Output is in CSV format: step_count,run_time(nanosec),sim_time(nanosec)
    #[arg(long, default_value_t = false)]
    benchmark: bool,
}

const BUFFER_SIZE: usize = 1 << 24;

fn main() {
    let args = Args::parse();

    let rng = match args.seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::try_from_rng(&mut rand::rng())
            .expect("failed to get a seed from OS entropy"),
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_handler = Arc::clone(&running);

    ctrlc::set_handler(move || {
        running_handler.store(false, Ordering::Relaxed);
        if args.logging {
            println!("Simulation terminated, stopping...");
        }
    })
    .unwrap();

    let (mut user_order_prod, user_order_cons) = HeapRb::<Order>::new(BUFFER_SIZE).split();
    let (market_event_prod, mut market_event_cons) =
        HeapRb::<MarketEvent>::new(BUFFER_SIZE).split();
    let (client_event_prod, mut client_event_cons) =
        HeapRb::<ClientEvent>::new(BUFFER_SIZE).split();
    if args.logging {
        println!("Initialized order queues");
    }

    let latency_settings = LatencyConfig {
        latency: args.sim_latency,
        jitter: SimJitter::from(args.sim_jitter),
    };
    let filename = "./test-file.bin";
    let mut source = match FileReplaySource::new(filename,64) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("{}",e);
            return;
        },
    };
    let mut poissony = PoissonSource::new(
            ConstantPoissonRate::new(args.order_rate),
            UniformTypeSelector::new(
                args.bid_rate,
                args.new_limit_rate,
                args.market_rate,
                args.cancel_rate,
                args.update_rate,
            ),
            GaussianOrderGenerator::new(args.avg_price, args.price_dev),
            rng.clone(),
        );
    // let generator = Box::new(move || source.next_event());
    let generator = SourceEnum::Poisson(poissony);
    let mut sim = Simulator::new(
        generator,
        SingleEventFeed::new(market_event_prod, client_event_prod),
        user_order_cons,
        latency_settings,
        rng.clone(),
        args.real_time,
    );
    if args.logging {
        println!("Spawned simulator");
    }

    let mut mold_engine = MoldEngine::start(Arc::clone(&running));
    let broadcast_running = Arc::clone(&running);
    let event_broadcast_thread = thread::spawn(move || {
        while broadcast_running.load(Ordering::Relaxed) {
            if let Some(order) = market_event_cons.try_pop() {
                mold_engine.push(order);
            }
        }
    });
    if args.logging {
        println!("MoldEngine started");
    }

    let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
    let gateway_running = Arc::clone(&running);
    let order_gateway_thread = thread::spawn(move || {
        let (mut engine, mut handler) = FixEngine::new(addr, "ENGINE01".to_owned()).unwrap();
        let engine_thread = thread::spawn(move || {
            engine.run();
        });
        while gateway_running.load(Ordering::Relaxed) {
            if let Some(order) = handler.get_order() {
                // TODO: Find a more elegant way to handle this
                while user_order_prod.try_push(order).is_err() {
                    if args.logging {
                        println!(
                            "OrderGateway failed to push an event into processing queue, buffer may be full"
                        );
                    }
                }
            }
            if let Some(client_event) = client_event_cons.try_pop() {
                let exec_report = ExecutionReport::from(client_event);
                let msg = FIXEvent {
                    comp_id: "".into(),
                    payload: FIXPayload::Report(ReportMessage::ExecutionReport(exec_report)),
                };
                handler.send_message(msg);
            }
        }
    });
    if args.logging {
        println!("FixEngine started");
        println!("Running...");
    }
    let mut sim_step_count: u128 = 0;
    let time = Instant::now();
    match args.count {
        Some(range) => {
            for _ in 0..range {
                #[allow(dead_code)]
                if let Err(msg) = sim.step() {
                    if args.logging {
                        println!("{}", msg);
                    }
                    break;
                }
                sim_step_count += 1;
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
        None => {
            while running.load(Ordering::Relaxed) {
                #[allow(dead_code)]
                if let Err(msg) = sim.step() {
                    if args.logging {
                        println!("{}", msg);
                    }
                    break;
                }
                sim_step_count += 1;
            }
        }
    }
    let elapsed = time.elapsed();
    running.store(false, Ordering::Relaxed);

    if args.logging && !args.benchmark {
        println!("Job finished");
        println!("Simulation covered {} steps", sim_step_count);
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
    } else if args.benchmark {
        println!("{},{},{}", sim_step_count, elapsed.as_nanos(), sim.time());
    }

    let _ = event_broadcast_thread.join();
    let _ = order_gateway_thread.join();
}
