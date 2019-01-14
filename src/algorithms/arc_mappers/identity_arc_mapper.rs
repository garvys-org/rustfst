use crate::algorithms::ArcMapper;
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper that returns its input.
pub struct IdentityArcMapper {}

impl<S: Semiring> ArcMapper<S> for IdentityArcMapper {
    fn arc_map(&mut self, _arc: &mut Arc<S>) {}

    fn final_weight_map(&mut self, _weight: &mut S) {}
}
