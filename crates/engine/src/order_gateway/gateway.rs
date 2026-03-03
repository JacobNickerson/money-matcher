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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{lob::order::Order};
    
    use std::net::SocketAddr;
    use ringbuf::{HeapProd, HeapCons, HeapRb, traits::*};

    #[test]
    fn test_make_gateway() {
        let (mut user_prod, mut user_cons) = HeapRb::<Order>::new(128).split();
        let (mut synth_prod, mut synth_cons) = HeapRb::<Order>::new(128).split();
        let (mut merge_prod, mut merge_cons) = HeapRb::<Order>::new(128).split();
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let gateway = OrderGateway::new(
            FixEngine::new(
                addr,
                user_prod,
            ).unwrap(),
            OrderMerger::new(
                synth_cons,
                user_cons,
                merge_prod,
                1024
            )
        ); 
    }
}