use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
mod fix;
mod lob;
mod moldudp64;

#[pymodule]
mod pyclient {
    use std::net::SocketAddr;

    use crate::{
        fix::client::{FixClient, FixClientHandler},
        lob::limit_order_book::OrderBook,
        moldudp64::client::MoldClient,
    };
    use mm_core::{
        fix_core::messages::{
            BusinessMessage, FIXEvent, FIXPayload,
            new_order_single::NewOrderSingle,
            types::{CustomerOrFirm, EncryptMethod, OpenClose, OrdType, PutOrCall, Side},
        },
        lob_core::{
            market_events::{MarketEvent, MarketEventType, TradeEvent},
            market_orders::{LimitOrder, Order, OrderSide, OrderType},
        },
    };
    use pyo3::prelude::*;
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};

    #[gen_stub_pyclass_enum]
    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PyOrderSide {
        Bid,
        Ask,
    }
    impl From<PyOrderSide> for OrderSide {
        fn from(value: PyOrderSide) -> Self {
            match value {
                PyOrderSide::Bid => OrderSide::Bid,
                PyOrderSide::Ask => OrderSide::Ask,
            }
        }
    }
    impl From<OrderSide> for PyOrderSide {
        fn from(value: OrderSide) -> Self {
            match value {
                OrderSide::Bid => PyOrderSide::Bid,
                OrderSide::Ask => PyOrderSide::Ask,
            }
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PyOrderType {
        inner: OrderType,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyOrderType {
        #[staticmethod]
        fn limit(qty: u64, price: u64) -> Self {
            Self {
                inner: OrderType::Limit { qty, price },
            }
        }

        #[staticmethod]
        fn market(qty: u64) -> Self {
            Self {
                inner: OrderType::Market { qty },
            }
        }

        #[staticmethod]
        fn update(old_id: u64, qty: u64, price: u64) -> Self {
            Self {
                inner: OrderType::Update { old_id, qty, price },
            }
        }

        #[staticmethod]
        fn cancel() -> Self {
            Self {
                inner: OrderType::Cancel {},
            }
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PyOrder {
        inner: Order,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyOrder {
        #[new]
        fn new(order_id: u64, side: PyOrderSide, timestamp: u64, kind: PyOrderType) -> Self {
            Self {
                inner: Order {
                    order_id,
                    side: OrderSide::from(side),
                    timestamp,
                    kind: kind.inner,
                },
            }
        }
    }
    impl From<PyOrder> for Order {
        fn from(value: PyOrder) -> Self {
            value.inner
        }
    }
    impl From<Order> for PyOrder {
        fn from(value: Order) -> Self {
            PyOrder { inner: value }
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PyLimitOrder {
        inner: LimitOrder,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyLimitOrder {
        #[new]
        fn new(order: PyOrder) -> Self {
            Self {
                inner: LimitOrder::new(Order::from(order)),
            }
        }
    }
    impl From<PyLimitOrder> for LimitOrder {
        fn from(value: PyLimitOrder) -> Self {
            value.inner
        }
    }
    impl From<LimitOrder> for PyLimitOrder {
        fn from(value: LimitOrder) -> Self {
            PyLimitOrder { inner: value }
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Debug, Clone, Copy)]
    struct PyMarketEventType {
        inner: MarketEventType,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyMarketEventType {
        #[staticmethod]
        fn l3(event: PyOrder) -> Self {
            Self {
                inner: MarketEventType::L3(Order::from(event)),
            }
        }
        #[staticmethod]
        fn trade(price: u64, quantity: u64, aggressor_side: PyOrderSide) -> Self {
            Self {
                inner: MarketEventType::Trade(TradeEvent {
                    price,
                    quantity,
                    aggressor_side: OrderSide::from(aggressor_side),
                }),
            }
        }
    }
    impl From<MarketEventType> for PyMarketEventType {
        fn from(value: MarketEventType) -> Self {
            Self { inner: value }
        }
    }
    impl From<PyMarketEventType> for MarketEventType {
        fn from(value: PyMarketEventType) -> Self {
            value.inner
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Copy, Clone, Debug)]
    struct PyMarketEvent {
        pub timestamp: u64,
        pub kind: PyMarketEventType,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyMarketEvent {
        #[new]
        fn new(timestamp: u64, kind: PyMarketEventType) -> Self {
            Self { timestamp, kind }
        }
    }
    impl From<MarketEvent> for PyMarketEvent {
        fn from(value: MarketEvent) -> Self {
            Self {
                timestamp: value.timestamp,
                kind: PyMarketEventType::from(value.kind),
            }
        }
    }
    impl From<PyMarketEvent> for MarketEvent {
        fn from(value: PyMarketEvent) -> Self {
            Self {
                timestamp: value.timestamp,
                kind: MarketEventType::from(value.kind),
            }
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    /// A stripped down version of the OrderBook. Directly manages its state
    /// via MarketEvents instead of handling matching logic, trade execution, etc.
    struct PyOrderBook {
        inner: OrderBook,
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl PyOrderBook {
        #[new]
        fn new() -> Self {
            Self {
                inner: OrderBook::default(),
            }
        }

        /// Accepts a market event and updates the state of the book
        pub fn process_event(&mut self, event: PyMarketEvent) {
            self.inner.process_event(MarketEvent::from(event));
        }

        /// Returns the best bidding price or None if there are no bids
        pub fn best_bid(&self) -> Option<u64> {
            self.inner.best_bid()
        }

        /// Returns a tuple of (best_bid_price,qty) or None if there are no bids
        pub fn best_bid_and_size(&self) -> Option<(u64, u64)> {
            self.inner.best_bid_and_size()
        }

        /// Returns the best asking price or None if there are no bids
        pub fn best_ask(&self) -> Option<u64> {
            self.inner.best_ask()
        }

        /// Returns a tuple of (best_ask_price,qty) or None if there are no bids
        pub fn best_ask_and_size(&self) -> Option<(u64, u64)> {
            self.inner.best_ask_and_size()
        }

        /// Returns an average of the best asking and bidding prices or None if there are no bids or no orders
        pub fn mid_price(&self) -> Option<f64> {
            self.inner.mid_price()
        }

        /// Returns the difference between the best asking and bidding prices or None if there are no bids or no orders
        pub fn spread(&self) -> Option<u64> {
            self.inner.spread()
        }

        /// Returns the quantity of a given price level on the specified side
        pub fn get_level(&self, price: u64, side: PyOrderSide) -> u64 {
            self.inner.get_level(price, OrderSide::from(side))
        }

        /// Returns the quantities of the top n price levels on the specified side
        pub fn get_top_levels(&self, side: PyOrderSide, n: usize) -> Vec<(u64, u64)> {
            self.inner.get_top_levels(OrderSide::from(side), n)
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    struct PyMoldClient {
        inner: MoldClient,
    }

    #[gen_stub_pymethods]
    #[pymethods]
    impl PyMoldClient {
        #[staticmethod]
        pub fn start() -> Self {
            Self {
                inner: MoldClient::start(),
            }
        }

        pub fn next_event(&mut self) -> Option<PyMarketEvent> {
            self.inner.next_event().map(PyMarketEvent::from)
        }
    }

    #[gen_stub_pyclass]
    #[pyclass]
    pub struct PyFixClient {
        handler: FixClientHandler,
    }

    #[gen_stub_pymethods]
    #[pymethods]
    impl PyFixClient {
        #[staticmethod]
        pub fn start(server_addr: String, comp_id: String, target_comp_id: String) -> Self {
            let addr: SocketAddr = server_addr.parse().unwrap();

            let (mut client, handler) =
                FixClient::new(addr, comp_id, target_comp_id, 10, EncryptMethod::None).unwrap();

            client.connect().unwrap();

            std::thread::spawn(move || {
                client.run();
            });

            Self { handler }
        }

        pub fn next_report(&mut self) -> Option<FIXEvent> {
            self.handler.next_report()
        }

        pub fn send_message(&mut self, payload: FIXPayload) {
            self.handler.send_message(payload);
        }

        pub fn send_generic_message(&mut self) {
            let order = NewOrderSingle {
                cl_ord_id: 1,
                handl_inst: 1,
                qty: 10,
                ord_type: OrdType::Limit,
                price: 666,
                side: Side::Buy,
                symbol: "OSISTRING".to_string(),
                transact_time: None,
                open_close: OpenClose::Open,
                security_type: "OPT".to_string(),
                put_or_call: PutOrCall::Call,
                strike_price: 10,
                customer_or_firm: CustomerOrFirm::Customer,
                maturity_day: 10,
            };

            let payload = FIXPayload::Business(BusinessMessage::NewOrderSingle(order));

            self.handler.send_message(payload);
        }
    }
}

define_stub_info_gatherer!(stub_info);
