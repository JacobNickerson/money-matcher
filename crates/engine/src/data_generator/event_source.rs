use rand::Rng;

use crate::data_generator::event_router::EventRouter;
use crate::data_generator::order_generators::OrderGenerator;
use crate::data_generator::rate_controllers::RateController;
use crate::data_generator::type_selectors::TypeSelector;

pub trait EventSource {
    fn next_event(&mut self);
}

/* Poisson */
pub struct PoissonSource<
    R: RateController,
    T: TypeSelector,
    G: OrderGenerator,
    E: EventRouter,
    N: Rng,
> {
    rate_controller: R,
    type_selector: T,
    order_generator: G,
    event_router: E,
    rng: N,
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, E: EventRouter, N: Rng>
    PoissonSource<R, T, G, E, N>
{
    pub fn new(
        rate_controller: R,
        type_selector: T,
        order_generator: G,
        event_router: E,
        rng: N,
    ) -> Self {
        Self {
            rate_controller,
            type_selector,
            order_generator,
            event_router,
            rng,
        }
    }
}
impl<R: RateController, T: TypeSelector, G: OrderGenerator, E: EventRouter, N: Rng> EventSource
    for PoissonSource<R, T, G, E, N>
{
    fn next_event(&mut self) {
        let dt = self.rate_controller.next_dt(&mut self.rng);
        let kind = self.type_selector.sample(&mut self.rng);
        let order = self.order_generator.generate(dt, kind, &mut self.rng);
        self.event_router.route_event(order);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_generator::{
        event_router::VectorStorageRouter, order_generators::GaussianOrderGenerator,
        rate_controllers::ConstantPoissonRate, type_selectors::UniformTypeSelector,
    };
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn poisson_generator() -> PoissonSource<
        ConstantPoissonRate,
        UniformTypeSelector,
        GaussianOrderGenerator,
        VectorStorageRouter,
        ChaCha8Rng,
    > {
        PoissonSource::new(
            ConstantPoissonRate::new(1_000_000.0),
            UniformTypeSelector::new(0.5, 0.4, 0.3, 0.2, 0.1),
            GaussianOrderGenerator::new(15.0, 1.0),
            VectorStorageRouter::new(),
            ChaCha8Rng::seed_from_u64(0),
        )
    }
    #[test]
    fn orders_are_monotonic_in_time() {
        let mut generator = poisson_generator();
        for _ in 0..1_000_000 {
            generator.next_event();
        }
        assert!(
            generator
                .event_router
                .events
                .windows(2)
                .all(|e| e[0].timestamp <= e[1].timestamp)
        );
    }
}
