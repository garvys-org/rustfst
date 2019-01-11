use crate::algorithms::{ArcMapper, ArcMapperMut};
use crate::semirings::Semiring;
use crate::Arc;

pub struct IdentityArcMapper {}

impl<S: Semiring> ArcMapper<S> for IdentityArcMapper {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S> {
        Arc::new(
            arc.ilabel,
            arc.olabel,
            self.weight_map(&arc.weight),
            arc.nextstate,
        )
    }

    fn weight_map(&mut self, weight: &S) -> S {
        weight.clone()
    }
}

impl<S: Semiring> ArcMapperMut<S> for IdentityArcMapper {
    fn arc_map_mut(&mut self, _arc: &mut Arc<S>) {}

    fn weight_map_mut(&mut self, _weight: &mut S) {}
}
