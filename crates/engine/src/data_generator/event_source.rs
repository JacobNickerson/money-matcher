use crate::data_generator::order_generators::OrderGenerator;
use crate::data_generator::rate_controllers::RateController;
use crate::data_generator::type_selectors::TypeSelector;
use memmap2::Mmap;
use mm_core::lob_core::market_orders::Order;
use rand::Rng;
use std::fs::File;
use std::io::Result;
use std::vec::Vec;

pub trait EventSource {
    fn next_event(&mut self) -> Option<Order>;
}

/* Poisson */
pub struct PoissonSource<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> {
    rate_controller: R,
    type_selector: T,
    order_generator: G,
    rng: N,
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> PoissonSource<R, T, G, N> {
    pub fn new(rate_controller: R, type_selector: T, order_generator: G, rng: N) -> Self {
        Self {
            rate_controller,
            type_selector,
            order_generator,
            rng,
        }
    }
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> EventSource
    for PoissonSource<R, T, G, N>
{
    fn next_event(&mut self) -> Option<Order> {
        let dt = self.rate_controller.next_dt(&mut self.rng);
        let kind = self.type_selector.sample(&mut self.rng);
        // TODO: Right now client_id is hard-coded as 0, but maybe it should be configurable
        Some(self.order_generator.generate(0, dt, kind, &mut self.rng))
    }
}

/// EventSource that replays orders from a binary file created by OrderLogger
/// Expects that binary file contains binary-serialized Orders
pub struct FileReplaySource {
    mmap: Mmap,
    file_size: usize,
    offset: usize,
    batch_size: usize,
    read_size: usize,
    buffer: Vec<Order>, 
    remaining: usize,
}
impl FileReplaySource {
    pub fn new(path: &str, batch_size: usize) -> Result<Self> {
        let file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;

        let this = Ok(Self {
            mmap: unsafe { Mmap::map(&file)? },
            file_size,
            batch_size,
            read_size: (batch_size / size_of::<Order>()) * size_of::<Order>(),
            offset: 0,
            buffer: Vec::with_capacity(batch_size),
            remaining: 0,
        });
        this
    }
    fn read_file(&mut self) {
        // TODO: Handle end of file
        let end = std::cmp::min(self.offset+self.read_size,self.file_size);
        let chunk_size = end-self.offset;
        let read_size = chunk_size * size_of::<Order>();
        let chunk = &self.mmap[self.offset..end]; 
        for (idx, rec_bytes) in chunk.chunks_exact(read_size).enumerate() {
            self.buffer[idx] = unsafe { *(rec_bytes.as_ptr() as *const Order) };
        }
        self.offset += read_size;
        self.remaining = chunk_size; 
    }

}
impl EventSource for FileReplaySource {
    fn next_event(&mut self) -> Option<Order> {
        if self.offset == self.file_size {
            return None;
        }
        if self.remaining == 0 {
            self.read_file();
        }
        let order = self.buffer[self.buffer.len()-self.remaining];
        self.remaining -= 1;
        Some(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_generator::{
        order_generators::GaussianOrderGenerator, rate_controllers::ConstantPoissonRate,
        type_selectors::UniformTypeSelector,
    };
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn poisson_generator()
    -> PoissonSource<ConstantPoissonRate, UniformTypeSelector, GaussianOrderGenerator, ChaCha8Rng>
    {
        PoissonSource::new(
            ConstantPoissonRate::new(1_000_000.0),
            UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
            GaussianOrderGenerator::new(15.0, 1.0),
            ChaCha8Rng::seed_from_u64(0),
        )
    }
    #[test]
    fn orders_are_monotonic_in_time() {
        let mut generator = poisson_generator();
        let mut events = Vec::with_capacity(1_000_000);
        for _ in 0..1_000_000 {
            events.push(generator.next_event());
        }
        assert!(events.windows(2).all(|e| e[0].timestamp <= e[1].timestamp));
    }
}
