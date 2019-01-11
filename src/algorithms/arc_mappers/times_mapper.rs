use crate::algorithms::{ArcMapper, ArcMapperMut};
use crate::semirings::Semiring;
use crate::Arc;

pub struct TimesMapper<W: Semiring> {
    to_multiply: W,
}

impl<S: Semiring> ArcMapper<S> for TimesMapper<S> {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S> {
        Arc::new(
            arc.ilabel,
            arc.olabel,
            self.weight_map(&arc.weight),
            arc.nextstate,
        )
    }

    fn weight_map(&mut self, weight: &S) -> S {
        weight.times(&self.to_multiply)
    }
}

impl<S: Semiring> ArcMapperMut<S> for TimesMapper<S> {
    fn arc_map_mut(&mut self, arc: &mut Arc<S>) {
        self.weight_map_mut(&mut arc.weight);
    }

    fn weight_map_mut(&mut self, weight: &mut S) {
        weight.times_mut(&self.to_multiply);
    }
}
