use crate::itch_core::messages::{
    add_order::AddOrder, order_cancel::OrderCancel, order_delete::OrderDelete,
    order_executed::OrderExecuted, order_executed_with_price::OrderExecutedWithPrice,
    order_replace::OrderReplace, test_benchmark::TestBenchmark,
};

pub mod add_order;
pub mod order_cancel;
pub mod order_delete;
pub mod order_executed;
pub mod order_executed_with_price;
pub mod order_replace;
pub mod test_benchmark;

pub const MESSAGE_TYPE_ADD_ORDER: u8 = b'A';
pub const MESSAGE_TYPE_ORDER_CANCEL: u8 = b'X';
pub const MESSAGE_TYPE_ORDER_DELETE: u8 = b'D';
pub const MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE: u8 = b'C';
pub const MESSAGE_TYPE_ORDER_EXECUTED: u8 = b'E';
pub const MESSAGE_TYPE_ORDER_REPLACE: u8 = b'U';
pub const MESSAGE_TYPE_TEST_BENCHMARK: u8 = b'b';

pub trait ItchMessage {
    fn set_tracking_number(&mut self, n: u16);
    fn set_stock_locate(&mut self, n: u16);
}

pub enum ItchEvent {
    AddOrder(AddOrder),
    OrderCancel(OrderCancel),
    OrderDelete(OrderDelete),
    OrderExecuted(OrderExecuted),
    OrderExecutedWithPrice(OrderExecutedWithPrice),
    OrderReplace(OrderReplace),
    TestBenchmark(TestBenchmark),
}
