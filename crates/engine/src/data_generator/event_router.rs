use crate::lob::order::Order;
use std::vec::Vec;

pub trait EventRouter {
    fn route_event(&mut self, order: Order);
}

pub struct VectorStorageRouter {
    pub events: Vec<Order>,
}
impl VectorStorageRouter {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}
impl EventRouter for VectorStorageRouter {
    fn route_event(&mut self, order: Order) {
        self.events.push(order);
    }
}
