use crate::algorithms::{ArcMapper, ArcMapperMut};
use crate::semirings::Semiring;
use crate::Arc;

pub struct PlusMapper<W: Semiring> {
    to_add: W,
}

impl<S: Semiring> ArcMapper<S> for PlusMapper<S> {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S> {
        Arc::new(
            arc.ilabel,
            arc.olabel,
            self.weight_map(&arc.weight),
            arc.nextstate,
        )
    }

    fn weight_map(&mut self, weight: &S) -> S {
        weight.plus(&self.to_add)
    }
}

impl<S: Semiring> ArcMapperMut<S> for PlusMapper<S> {
    fn arc_map_inplace(&mut self, arc: &mut Arc<S>) {
        self.weight_map_inplace(&mut arc.weight);
    }

    fn weight_map_inplace(&mut self, weight: &mut S) {
        weight.plus_mut(&self.to_add);
    }
}
