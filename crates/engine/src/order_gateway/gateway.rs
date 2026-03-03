use rand::seq::SliceRandom;

use crate::{fix::engine::FixEngine, order_gateway::merger::OrderMerger};
use std::thread::{self, JoinHandle};

pub struct OrderGateway {
    pub polling_thread: JoinHandle<()>,
    pub merging_thread: JoinHandle<()>,
}
impl OrderGateway {
    pub fn new(mut poller: FixEngine, mut merger: OrderMerger) -> Self {
        let polling_thread = thread::spawn(move || {
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