use rand::seq::SliceRandom;

use crate::order_gateway::{merger::OrderMerger, poller::OrderPoller};
use std::thread::{self, JoinHandle};

pub struct OrderGateway {
    pub polling_thread: JoinHandle<()>,
    pub merging_thread: JoinHandle<()>,
}
impl OrderGateway {
    pub fn new(mut poller: OrderPoller, mut merger: OrderMerger) -> Self {
        let polling_thread = thread::spawn(move || {
                // TODO: Replace placeholder with actual polling loop
                poller.run();
            });
        let merging_thread = thread::spawn(move || {
				merger.run();
			});

        Self {
            polling_thread,
            merging_thread,
        }
    }
}
