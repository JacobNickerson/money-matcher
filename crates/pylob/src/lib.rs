use pyo3::prelude::*;
mod limit_order_book;

#[pymodule]
mod pylob {
    use crate::limit_order_book::OrderBook;
    use engine::lob::{
        market_events::{LiquidityFlag, MarketEvent, MarketEventType, TradeEvent},
        order::{LimitOrder, Order, OrderSide, OrderType},
    };
    use pyo3::prelude::*;

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

    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PyOrderType {
        inner: OrderType,
    }
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

    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PyOrder {
        inner: Order,
    }
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

    #[pyclass]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PyLimitOrder {
        inner: LimitOrder,
    }
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

    #[pyclass]
    #[derive(Debug, Clone, Copy)]
    struct PyMarketEventType {
        inner: MarketEventType,
    }
    #[pymethods]
    impl PyMarketEventType {
        #[staticmethod]
        fn l3(event: PyLimitOrder) -> Self {
            Self {
                inner: MarketEventType::L3(LimitOrder::from(event)),
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

    #[pyclass]
    #[derive(Copy, Clone, Debug)]
    struct PyMarketEvent {
        pub timestamp: u64,
        pub kind: PyMarketEventType,
    }
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

    #[pyclass]
    /// A stripped down version of the OrderBook. Directly manages its state
    /// via MarketEvents instead of handling matching logic, trade execution, etc.
    struct PyOrderBook {
        inner: OrderBook,
    }
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
}
