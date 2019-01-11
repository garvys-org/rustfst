use crate::algorithms::{ArcMapper, ArcMapperMut};
use crate::semirings::Semiring;
use crate::Arc;

pub struct RmWeightMapper {}

impl<S: Semiring> ArcMapper<S> for RmWeightMapper {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S> {
        Arc::new(
            arc.ilabel,
            arc.olabel,
            self.weight_map(&arc.weight),
            arc.nextstate,
        )
    }

    fn weight_map(&mut self, weight: &S) -> S {
        if weight.is_zero() {
            weight.clone()
        } else {
            S::one()
        }
    }
}

impl<S: Semiring> ArcMapperMut<S> for RmWeightMapper {
    fn arc_map_inplace(&mut self, arc: &mut Arc<S>) {
        self.weight_map_inplace(&mut arc.weight);
    }

    fn weight_map_inplace(&mut self, weight: &mut S) {
        if !weight.is_zero() {
            weight.set_value(S::one().value())
        }
    }
}
