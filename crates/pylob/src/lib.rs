use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod pylob {
    use engine::lob::{
        limitorderbook::OrderBook,
        market_events::NullFeeds,
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
            Order {
                order_id: value.inner.order_id,
                side: value.inner.side,
                timestamp: value.inner.timestamp,
                kind: value.inner.kind,
            }
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
        fn new(order: PyOrder, qty: u64, price: u64) -> Self {
            Self {
                inner: LimitOrder::new(Order::from(order), qty, price),
            }
        }
    }
    impl From<PyLimitOrder> for LimitOrder {
        fn from(value: PyLimitOrder) -> Self {
            LimitOrder {
                order_id: value.inner.order_id,
                side: value.inner.side,
                status: value.inner.status,
                qty: value.inner.qty,
                price: value.inner.price,
            }
        }
    }
    impl From<LimitOrder> for PyLimitOrder {
        fn from(value: LimitOrder) -> Self {
            PyLimitOrder { inner: value }
        }
    }

    #[pyclass]
    struct PyOrderBook {
        inner: OrderBook<NullFeeds>,
    }
    #[pymethods]
    impl PyOrderBook {
        #[new]
        pub fn new() -> Self {
            Self {
                inner: OrderBook::<NullFeeds>::new(NullFeeds {}),
            }
        }

        pub fn process_order(&mut self, order: PyOrder, time: u64) -> Option<PyLimitOrder> {
            self.inner
                .process_order(Order::from(order), time)
                .map(PyLimitOrder::from)
        }
    }
}
