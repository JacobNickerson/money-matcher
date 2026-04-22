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

use crate::data_generator::event_source::{
    ConstantPoissonSource, EventSource, FileReplaySource, SourceFunction,
};
use crate::data_generator::order_generators::GaussianOrderGenerator;
use crate::data_generator::rate_controllers::ConstantPoissonRate;
use crate::data_generator::type_selectors::UniformTypeSelector;
use crate::fix::engine::FixEngine;
use crate::moldudp64::engine::MoldEngine;
use crate::simulator::DynamicSimulator;
use crate::simulator::latency_config::{LatencyConfig, SimJitter};

use crate::cli_args::{Args, EventSourceType, validate};
use crate::event_logger::BinaryLogger;

mod cli_args;
mod data_generator;
mod event_logger;
mod fix;
mod moldudp64;
mod simulator;

const BUFFER_SIZE: usize = 1 << 24;

fn main() {
    let args = Args::parse();
    if let Err(msg) = validate(&args) {
        eprintln!("{}", msg);
        return;
    }

    if args.logging {
        println!("Setting RNG seed");
    }
    let rng = match args.seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::try_from_rng(&mut rand::rng())
            .expect("failed to get a seed from OS entropy"),
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_handler = Arc::clone(&running);
    ctrlc::set_handler(move || {
        if args.logging {
            println!("Simulation terminated, stopping...");
        }
        running_handler.store(false, Ordering::Relaxed);
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
        jitter: SimJitter::from(&args),
    };

    let source = match args.event_source {
        EventSourceType::Poisson {
            count,
            order_rate,
            bid_rate,
            new_limit_rate,
            cancel_rate,
            market_rate,
            update_rate,
            bid_avg_price,
            bid_price_dev,
            ask_avg_price,
            ask_price_dev,
        } => {
            let mut source = ConstantPoissonSource::new(
                ConstantPoissonRate::new(order_rate),
                UniformTypeSelector::new(
                    bid_rate,
                    new_limit_rate,
                    market_rate,
                    cancel_rate,
                    update_rate,
                ),
                GaussianOrderGenerator::new(bid_avg_price,bid_price_dev,ask_avg_price,ask_price_dev),
                rng.clone(),
                count,
            );
            SourceFunction::new(Box::new(move || source.next_event()))
        }
        EventSourceType::File {
            file_name,
            batch_size,
        } => {
            let mut source = match FileReplaySource::new(&file_name, batch_size) {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            SourceFunction::new(Box::new(move || source.next_event()))
        }
    };

    let mut sim = DynamicSimulator::new(
        source,
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
        let engine_running = Arc::clone(&gateway_running);
        let engine_thread = thread::spawn(move || {
            engine.run(Arc::clone(&engine_running));
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
        if args.logging {
            println!("FixEngine shutting down...");
        }
        let _ = engine_thread.join();
    });
    if args.logging {
        println!("FixEngine started");
        println!("Running...");
    }

    let mut sim_step_count: u128 = 0;
    let time = Instant::now();

    let mut logger = BinaryLogger::new("test.bin", 4).expect("Failed to create logger");

    while running.load(Ordering::Relaxed) {
        #[allow(dead_code)]
        let res = sim.step();
        if let Err(msg) = res {
            if args.logging {
                println!("{}", msg);
            }
            break;
        } else if let Ok(event) = res
            && args.logging && let Err(msg) = logger.log_order(&event) {
                println!("{}", msg)
            }
        sim_step_count += 1;
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
