pub mod latency_config;

use crate::data_generator::event_source::{EventSource, SourceEnum, SourceFunction};
use crate::simulator::latency_config::LatencyConfig;
use engine::limit_order_book::OrderBook;
use mm_core::lob_core::{market_events::EventSink, market_orders::Order};
use rand::Rng;
use ringbuf::{HeapCons, traits::*};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

const USER_ORDER_INGRESS: usize = 1024;
const SIM_HEAP_CAPACITY: usize = USER_ORDER_INGRESS * 10;

/// Represents current simulation time in nanoseconds
pub type SimTime = u64;

/// Object that owns the simulation, responsible for managing simulation time
pub struct Simulator<E: EventSource, S: EventSink, R: Rng> {
    time: SimTime,
    limit_order_book: OrderBook<S>,
    orders: BinaryHeap<Order>,
    source: E,
    user_orders: HeapCons<Order>,
    user_order_buffer: Vec<Order>,
    id_counter: u64,
    latency_settings: LatencyConfig,
    rng: R,
    real_time: Instant,
    is_real_time: bool,
}
impl<E: EventSource, S: EventSink, R: Rng> Simulator<E, S, R> {
    pub fn new(
        source: E,
        event_sink: S,
        user_orders: HeapCons<Order>,
        latency_settings: LatencyConfig,
        rng: R,
        is_real_time: bool,
    ) -> Self {
        Self {
            time: 0,
            limit_order_book: OrderBook::new(event_sink),
            orders: BinaryHeap::with_capacity(SIM_HEAP_CAPACITY),
            latency_settings,
            source,
            user_orders,
            user_order_buffer: vec![Order::default(); USER_ORDER_INGRESS],
            id_counter: 0,
            rng,
            real_time: Instant::now(),
            is_real_time,
        }
    }
    /// Takes a single step in the simulation. Generates a single order from its held order source,
    /// drains a batch of user orders, inserts them all into a heap, and selects the one with the lowest timestamp
    pub fn step(&mut self) -> Result<Order, String> {
        self.drain_user_orders();
        if let Some(synth_order) = self.generate_single_order() {
            self.orders.push(synth_order);
            let mut event = self.orders.pop().unwrap();
            event.order_id = self.id_counter;
            self.id_counter += 1;
            if self.is_real_time {
                self.pace(event.timestamp);
            }
            self.process_event(event);
            Ok(event)
        } else {
            Err("Reached end of event stream".to_string())
        }
    }
    /// Getter for the current simulation time
    pub fn time(&self) -> SimTime {
        self.time
    }
    /// Attempts to read a batch of user orders from the queue and inserts them into the order heap
    fn drain_user_orders(&mut self) {
        for i in 0..self.user_orders.pop_slice(&mut self.user_order_buffer) {
            let mut order = self.user_order_buffer[i];
            order.timestamp = self.time
                + self.latency_settings.latency
                + self.latency_settings.jitter.sample(&mut self.rng);
            self.orders.push(order)
        }
    }
    /// Generates a single event from the artificial event source
    fn generate_single_order(&mut self) -> Option<Order> {
        self.source.next_event()
    }
    /// Update the simulation time to the timestamp of the order, and pass the order into the limit order book
    fn process_event(&mut self, event: Order) {
        self.time = event.timestamp;
        self.limit_order_book.process_order(event);
    }
    /// Wait until wall clock time reaches the current simulation time
    fn pace(&self, next_event_time: SimTime) {
        let real_time_delta =
            next_event_time.saturating_sub(self.real_time.elapsed().as_nanos() as u64);

        std::thread::sleep(Duration::from_nanos(real_time_delta));
    }
}
/// A specific typedef of Simulator, where the EventSource is a struct that wraps around a function pointer
/// This allows the source type to be picked dynamically at run-time, but comes with a performance penalty for
/// virtual calls
pub type DynamicSimulator<S, R> = Simulator<SourceFunction, S, R>;
/// A specific typedef of Simulator, where the EventSource is an enum that contains a limited subset of EventSource types
/// This allows the source type to be picked dynamically at run-time, but only from the limited subset included in the enum
/// The performance penalty of this is negligible as long as the enum does not encompass too many types
pub type EnumSimulator<S, R> = Simulator<SourceEnum, S, R>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_generator::{
            event_source::RandomSource, order_generators::GaussianOrderGenerator,
            rate_controllers::ConstantPoissonRate, type_selectors::UniformTypeSelector,
        },
        simulator::latency_config::SimJitter,
    };
    use mm_core::lob_core::market_events::{ClientEvent, MarketEvent, NullFeeds, SingleEventFeed};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use ringbuf::HeapRb;

    #[test]
    fn simulator_time_monotonic() {
        let (_, user_order_cons) = HeapRb::<Order>::new(SIM_HEAP_CAPACITY).split();
        let mut sim = Simulator::new(
            RandomSource::new(
                ConstantPoissonRate::new(100_000.0),
                UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
                GaussianOrderGenerator::new(150.0, 30.0, 150.0, 30.0),
                ChaCha8Rng::seed_from_u64(0),
                None,
            ),
            NullFeeds {}, // use this since nothing is draining the market events
            user_order_cons,
            LatencyConfig {
                latency: 0,
                jitter: SimJitter::None,
            },
            ChaCha8Rng::seed_from_u64(67),
            false,
        );
        let _ = sim.step();
        let mut prev_time: SimTime = 0;
        while sim.time < 1_000_000_000 {
            // run for a full simulated second
            let _ = sim.step();
            assert!(
                sim.time >= prev_time,
                "Sim time: {:?}; Prev time: {:?}",
                sim.time,
                prev_time
            );
            prev_time = sim.time;
        }
    }

    #[test]
    fn event_time_monotonic() {
        let (_, user_order_cons) = HeapRb::<Order>::new(SIM_HEAP_CAPACITY).split();
        let (market_event_prod, mut market_event_cons) =
            HeapRb::<MarketEvent>::new(1 << 24).split();
        let (client_event_prod, mut client_event_cons) =
            HeapRb::<ClientEvent>::new(1 << 24).split();
        let mut sim = Simulator::new(
            RandomSource::new(
                ConstantPoissonRate::new(100_000.0),
                UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
                GaussianOrderGenerator::new(150.0, 30.0, 150.0, 30.0),
                ChaCha8Rng::seed_from_u64(0),
                None,
            ),
            SingleEventFeed::new(market_event_prod, client_event_prod),
            user_order_cons,
            LatencyConfig {
                latency: 0,
                jitter: SimJitter::None,
            },
            ChaCha8Rng::seed_from_u64(67),
            false,
        );
        for _ in 0..5_000 {
            let _ = sim.step();
        }
        let mut prev_time = market_event_cons.try_pop().unwrap().timestamp;
        let mut saw_greater_than_zero = false;
        while let Some(event) = market_event_cons.try_pop() {
            assert!(event.timestamp >= prev_time);
            saw_greater_than_zero = event.timestamp > 0;
            prev_time = event.timestamp;
        }
        assert!(saw_greater_than_zero);

        saw_greater_than_zero = false;
        prev_time = client_event_cons.try_pop().unwrap().timestamp;
        while let Some(event) = client_event_cons.try_pop() {
            assert!(event.timestamp >= prev_time);
            saw_greater_than_zero = event.timestamp > 0;
            prev_time = event.timestamp;
        }
        assert!(saw_greater_than_zero);
    }
}
