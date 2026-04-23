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
use std::time::{Duration, Instant};

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
use crate::event_recorder::{BinaryRecorder, RecorderEnum, RecorderType, TextRecorder};
use crate::logging::log;

mod cli_args;
mod data_generator;
mod event_recorder;
mod fix;
mod logging;
mod moldudp64;
mod simulator;

const BUFFER_SIZE: usize = 1 << 24;

fn main() {
    let args = Args::parse();
    if let Err(msg) = validate(&args) {
        eprintln!("{}", msg);
        return;
    }
    logging::set_enabled(args.logging);

    log("Setting RNG seed");
    let rng = match args.seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::try_from_rng(&mut rand::rng())
            .expect("failed to get a seed from OS entropy"),
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_handler = Arc::clone(&running);
    ctrlc::set_handler(move || {
        log("Simulation terminated due to user intervention, stopping...");
        running_handler.store(false, Ordering::Relaxed);
    })
    .unwrap();

    let (mut user_order_prod, user_order_cons) = HeapRb::<Order>::new(BUFFER_SIZE).split();
    let (market_event_prod, mut market_event_cons) =
        HeapRb::<MarketEvent>::new(BUFFER_SIZE).split();
    let (client_event_prod, mut client_event_cons) =
        HeapRb::<ClientEvent>::new(BUFFER_SIZE).split();
    let (mut recorder_prod, recorder_cons) = match args.record_type {
        Some(_) => {
            let (recorder_prod, recorder_cons) = HeapRb::<Order>::new(BUFFER_SIZE).split();
            (Some(recorder_prod), Some(recorder_cons))
        }
        None => (None, None),
    };
    log("Initialized SPSC queues");

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
                GaussianOrderGenerator::new(
                    bid_avg_price,
                    bid_price_dev,
                    ask_avg_price,
                    ask_price_dev,
                ),
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
    log("Spawned simulator");

    let mold_ready = Arc::new(AtomicBool::new(false));
    let ready = Arc::clone(&mold_ready);
    let mut mold_engine = MoldEngine::start(Arc::clone(&running));
    let broadcast_running = Arc::clone(&running);
    let event_broadcast_thread = thread::spawn(move || {
        ready.store(true, Ordering::Release);
        log("MoldEngine started");
        while broadcast_running.load(Ordering::Relaxed) {
            if let Some(order) = market_event_cons.try_pop() {
                mold_engine.push(order);
            }
        }
        thread::sleep(Duration::from_millis(5)); // Let finish before terminating 
        log("MoldEngine flushing remaining events...");
        while let Some(order) = market_event_cons.try_pop() {
            mold_engine.push(order);
        }
        log("MoldEngine shutting down");
    });

    let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();

    let gateway_running = Arc::clone(&running);
    let order_gateway_ready = Arc::new(AtomicBool::new(false));
    let ready = Arc::clone(&order_gateway_ready);

    let order_gateway_thread = thread::spawn(move || {
        let (mut engine, mut handler) = FixEngine::new(addr, "ENGINE01".to_owned()).unwrap();
        // Clone atomic to denote that the system is running
        // One atomic denotes if the ENGINE thread is ready
        // Once that atomic is true, we say that the whole gateway thread is ready
        // While system is running, run
        let engine_running = Arc::clone(&gateway_running);
        let engine_ready = Arc::new(AtomicBool::new(false));
        let engine_thread_ready = Arc::clone(&engine_ready); // Wow thats a lot of atomics
        let engine_thread = thread::spawn(move || {
            // I guess that happens when you have nested threads needing synchronization >_>
            engine.run(engine_thread_ready, engine_running);
        });
        while !engine_ready.load(Ordering::Acquire) {}
        ready.store(true, Ordering::Release);
        log("FixEngine started");
        while gateway_running.load(Ordering::Relaxed) {
            if let Some(order) = handler.get_order() {
                // TODO: Find a more elegant way to handle this
                while user_order_prod.try_push(order).is_err() {
                    log(
                        "OrderGateway failed to push an event into processing queue, buffer may be full",
                    );
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
        thread::sleep(Duration::from_millis(5));
        log("FixEngine flushing remaining execution reports...");
        while let Some(client_event) = client_event_cons.try_pop() {
            // Flush remaining ExecutionReports, but don't accept any more orders
            let exec_report = ExecutionReport::from(client_event);
            let msg = FIXEvent {
                comp_id: "".into(),
                payload: FIXPayload::Report(ReportMessage::ExecutionReport(exec_report)),
            };
            handler.send_message(msg);
        }
        let _ = engine_thread.join();
        log("FixEngine stopped");
    });

    let recorder_ready = Arc::new(AtomicBool::new(false));
    let ready = Arc::clone(&recorder_ready);
    let recorder_running = Arc::clone(&running);
    let recorder_thread = match args.record_type {
        Some(recorder_type) => Some(thread::spawn(move || {
            let mut recorder = match recorder_type {
                RecorderType::Binary => RecorderEnum::Binary(
                    BinaryRecorder::new(args.record_file.as_str(), args.record_batch_size)
                        .expect("Failed to create recorder"),
                ),
                RecorderType::Text => RecorderEnum::Text(
                    TextRecorder::new(args.record_file.as_str(), args.record_batch_size)
                        .expect("Failed to create recorder"),
                ),
            };
            let mut recorder_cons = recorder_cons.unwrap();
            ready.store(true, Ordering::Release);
            log("Recorder started");
            while recorder_running.load(Ordering::Relaxed) {
                if let Some(event) = recorder_cons.try_pop()
                    && let Err(msg) = recorder.record_event(event)
                {
                    log(format!("{}", msg).as_str());
                }
            }
            thread::sleep(Duration::from_millis(5)); // Let finish before terminating 
            log("Recorder flushing remaining events...");
            while let Some(event) = recorder_cons.try_pop() {
                if let Err(msg) = recorder.record_event(event) {
                    log(format!("{}", msg).as_str());
                }
            }
            log("Recorder stopped");
            let _ = recorder.shutdown();
        })),

        None => {
            ready.store(true, Ordering::Release);
            None
        }
    };

    let mut sim_step_count: u128 = 0;

    while !mold_ready.load(Ordering::Acquire)
        || !order_gateway_ready.load(Ordering::Acquire)
        || !recorder_ready.load(Ordering::Acquire)
    {
        // Wait for all the threads to be ready
    }
    log("Running...");
    let time = Instant::now();

    while running.load(Ordering::Relaxed) {
        #[allow(dead_code)]
        match sim.step() {
            Ok(event) => {
                if let Some(log_queue) = &mut recorder_prod
                    && let Err(_) = log_queue.try_push(event)
                {
                    log("failed to log an order, queue may be full");
                }
            }
            Err(msg) => {
                log(&msg);
                break;
            }
        }
        sim_step_count += 1;
    }
    let elapsed = time.elapsed();
    running.store(false, Ordering::Relaxed);

    if !args.benchmark {
        log("Job finished");
        log(format!("Simulation covered {} steps", sim_step_count).as_str());
        log(format!(
            "Sim time: {}s ({}ns)",
            sim.time() as f64 / 1_000_000_000.0,
            sim.time()
        )
        .as_str());
        log(format!(
            "Real time: {}s ({}ns)",
            elapsed.as_nanos() as f64 / 1_000_000_000.0,
            elapsed.as_nanos()
        )
        .as_str());
    } else {
        println!("{},{},{}", sim_step_count, elapsed.as_nanos(), sim.time());
    }

    let _ = event_broadcast_thread.join();
    let _ = order_gateway_thread.join();
    if let Some(t) = recorder_thread {
        let _ = t.join();
    }
}
