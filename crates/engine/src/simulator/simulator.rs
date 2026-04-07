use crate::data_generator::event_source::EventSource;
use crate::lob::limitorderbook::OrderBook;
use crate::simulator::latency_config::LatencyConfig;
use mm_core::lob_core::{market_events::EventSink, market_orders::Order};
use rand::Rng;
use ringbuf::{HeapCons, traits::*};
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration,Instant};

const USER_ORDER_INGRESS: usize = 1024;
const SIM_HEAP_CAPACITY: usize = USER_ORDER_INGRESS * 10;

/// Represents current simulation time in nanoseconds
pub type SimTime = u64;

/// Object that owns the simulation, responsible for managing simulation time
pub struct Simulator<E: EventSource, S: EventSink, R: Rng> {
    time: SimTime,
    limit_order_book: OrderBook<S>,
    orders: BinaryHeap<Order>,
    order_generator: E,
    user_orders: HeapCons<Order>,
    id_counter: u64,
    latency_settings: HashMap<u64, LatencyConfig>, // TODO: type def for identifier?
    rng: R,
    real_time: Instant,
    is_real_time: bool,
}
impl<E: EventSource, S: EventSink, R: Rng> Simulator<E, S, R> {
    pub fn new(order_generator: E, event_sink: S, user_orders: HeapCons<Order>, rng: R, is_real_time: bool) -> Self {
        Self {
            time: 0,
            limit_order_book: OrderBook::new(event_sink),
            orders: BinaryHeap::with_capacity(SIM_HEAP_CAPACITY),
            latency_settings: HashMap::new(),
            order_generator,
            user_orders,
            id_counter: 0,
            rng,
            real_time: Instant::now(),
            is_real_time,
        }
    }
    pub fn step(&mut self) {
        // TODO: Tune the ingress_value
        self.drain_user_orders(USER_ORDER_INGRESS);
        // TODO: Evaluate this is correct and doesn't cause issues
        let synth_order = self.generate_single_order();
        self.orders.push(synth_order);
        let event = self.orders.pop().unwrap();
        if self.is_real_time {
            self.pace(event.timestamp);
        }
        self.process_event(event);
    }
    pub fn time(&self) -> SimTime {
        self.time
    }
    fn drain_user_orders(&mut self, ingress_size: usize) {
        for _ in 0..ingress_size {
            if let Some(mut order) = self.user_orders.try_pop() {
                let ind: u64 = 0; // TODO: Attach some type of session identifier to each user message
                let cfg = self.latency_settings[&ind];
                order.timestamp = self.time + cfg.latency + cfg.jitter.sample(&mut self.rng);
                order.order_id = self.id_counter;
                self.id_counter += 1;
                self.orders.push(order)
            }
        }
    }
    fn generate_single_order(&mut self) -> Order {
        let mut event = self.order_generator.next_event();
        event.order_id = self.id_counter;
        self.id_counter += 1;
        event
    }
    fn generate_orders_til(&mut self, horizon: SimTime) {
        loop {
            let event = self.generate_single_order();
            self.orders.push(event);
            if event.timestamp > horizon {
                break;
            }
        }
    }
    fn process_event(&mut self, event: Order) {
        self.time = event.timestamp;
        self.limit_order_book.process_order(event);
    }
    fn pace(&self, next_event_time: SimTime) {
        let real_time_delta = next_event_time.saturating_sub(self.real_time.elapsed().as_nanos() as u64); 

        std::thread::sleep(Duration::from_nanos(real_time_delta));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_generator::{
        event_source::PoissonSource, order_generators::GaussianOrderGenerator,
        rate_controllers::ConstantPoissonRate, type_selectors::UniformTypeSelector,
    };
    use mm_core::lob_core::market_events::{ClientEvent, MarketEvent, NullFeeds, SingleEventFeed};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use ringbuf::HeapRb;
    use std::time::Instant;

    #[test]
    fn simulator_time_monotonic() {
        let (_, user_order_cons) = HeapRb::<Order>::new(SIM_HEAP_CAPACITY).split();
        let mut sim = Simulator::new(
            PoissonSource::new(
                ConstantPoissonRate::new(100_000.0),
                UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
                GaussianOrderGenerator::new(150.0, 30.0),
                ChaCha8Rng::seed_from_u64(0),
            ),
            NullFeeds {}, // use this since nothing is draining the market events
            user_order_cons,
            ChaCha8Rng::seed_from_u64(67),
            false,
        );
        sim.step();
        let mut prev_time: SimTime = 0;
        while sim.time < 1_000_000_000 {
            // run for a full simulated second
            sim.step();
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
            PoissonSource::new(
                ConstantPoissonRate::new(100_000.0),
                UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
                GaussianOrderGenerator::new(150.0, 30.0),
                ChaCha8Rng::seed_from_u64(0),
            ),
            SingleEventFeed::new(market_event_prod, client_event_prod),
            user_order_cons,
            ChaCha8Rng::seed_from_u64(67),
            false,
        );
        // for _ in 0..100_000 {
        for _ in 0..25 {
            sim.step();
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
