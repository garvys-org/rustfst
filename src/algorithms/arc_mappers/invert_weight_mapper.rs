use crate::algorithms::{ArcMapper, ArcMapperMut};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::Arc;

pub struct InvertWeightMapper {}

impl<S: WeaklyDivisibleSemiring> ArcMapper<S> for InvertWeightMapper {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S> {
        Arc::new(
            arc.ilabel,
            arc.olabel,
            self.weight_map(&arc.weight),
            arc.nextstate,
        )
    }

    fn weight_map(&mut self, weight: &S) -> S {
        weight.inverse()
    }
}

impl<S: WeaklyDivisibleSemiring> ArcMapperMut<S> for InvertWeightMapper {
    fn arc_map_mut(&mut self, arc: &mut Arc<S>) {
        self.weight_map_mut(&mut arc.weight);
    }

    fn weight_map_mut(&mut self, weight: &mut S) {
        weight.inverse_mut();
    }
}
