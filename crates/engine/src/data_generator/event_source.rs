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
