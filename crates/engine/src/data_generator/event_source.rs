use crate::data_generator::order_generators::{GaussianOrderGenerator, OrderGenerator};
use crate::data_generator::rate_controllers::{ConstantPoissonRate, RateController};
use crate::data_generator::type_selectors::{TypeSelector, UniformTypeSelector};
use mm_core::lob_core::market_orders::Order;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fs::File;
use std::io::{BufReader, Read};
use std::vec::Vec;
use std::convert::Infallible;
use rkyv::{Archived, Deserialize, access};

pub trait EventSource {
    fn next_event(&mut self) -> Option<Order>;
}
pub enum SourceEnum {
    Poisson(ConstantPoissonSource),
    File(FileReplaySource),
}
impl EventSource for SourceEnum {
    fn next_event(&mut self) -> Option<Order> {
        match self {
            SourceEnum::Poisson(this) => this.next_event(),
            SourceEnum::File(this) => this.next_event(),
        }
    }
}
pub struct SourceFunction {
    func: Box<dyn FnMut() -> Option<Order>>,
}
impl SourceFunction {
    pub fn new(func: Box<dyn FnMut() -> Option<Order>>) -> Self {
        Self { func }
    }
}
impl EventSource for SourceFunction {
    fn next_event(&mut self) -> Option<Order> {
        (self.func)()
    }
}

/* Poisson */
pub struct PoissonSource<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> {
    rate_controller: R,
    type_selector: T,
    order_generator: G,
    rng: N,
    limit: Option<u64>,
    count: u64,
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> PoissonSource<R, T, G, N> {
    pub fn new(
        rate_controller: R,
        type_selector: T,
        order_generator: G,
        rng: N,
        limit: Option<u64>,
    ) -> Self {
        Self {
            rate_controller,
            type_selector,
            order_generator,
            rng,
            limit,
            count: 0,
        }
    }
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, N: Rng> EventSource
    for PoissonSource<R, T, G, N>
{
    fn next_event(&mut self) -> Option<Order> {
        if let Some(limit) = self.limit
            && self.count >= limit
        {
            return None;
        }
        self.count += 1;
        let dt = self.rate_controller.next_dt(&mut self.rng);
        let kind = self.type_selector.sample(&mut self.rng);
        Some(self.order_generator.generate(0, dt, kind, &mut self.rng))
    }
}
pub type ConstantPoissonSource =
    PoissonSource<ConstantPoissonRate, UniformTypeSelector, GaussianOrderGenerator, ChaCha8Rng>;

/// EventSource that replays orders from a binary file created by OrderLogger
/// Expects that binary file contains binary-serialized Orders
pub struct FileReplaySource {
    reader: BufReader<File>,
    batch_size: usize,
    order_buf: Vec<Order>,
    len_buf: [u8; 8], 
    read_buf: [u8; 64],
}
impl FileReplaySource {
    pub fn new(path: &str, batch_size: usize) -> std::io::Result<Self> {
        let file = File::open(path)?;

        Ok(Self {
            reader: BufReader::new(file),
            batch_size,
            order_buf: Vec::with_capacity(batch_size),
            len_buf: [0; 8],
            read_buf: [0; 64],
        })
    }
    fn read_file(&mut self) {
        for i in 0..self.batch_size {
            if self.reader.read_exact(&mut self.len_buf).is_err() {
                return; // EOF
            }
            let len = u64::from_le_bytes(self.len_buf) as usize;
            self.reader.read_exact(&mut self.read_buf[0..len]);
            // let archived = access::<Archived<Order>,_>(&self.read_buf[0..len]).unwrap();
            // self.order_buf[i] = archived.deserialize(&mut Infallible).unwrap();
        }
    }
}
impl EventSource for FileReplaySource {
    fn next_event(&mut self) -> Option<Order> {
        // if self.offset == self.file_size {
        //     return None;
        // }
        // if self.remaining == 0 {
        //     self.read_file();
        // }
        // let order = self.buffer[self.buffer.len() - self.remaining];
        // self.remaining -= 1;
        None
        // Some(order)
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
            None,
        )
    }
    #[test]
    fn orders_are_monotonic_in_time() {
        let mut generator = poisson_generator();
        let mut events = Vec::with_capacity(1_000_000);
        for _ in 0..1_000_000 {
            events.push(generator.next_event());
        }
        assert!(
            events
                .windows(2)
                .all(|e| e[0].unwrap().timestamp <= e[1].unwrap().timestamp)
        );
    }
}
