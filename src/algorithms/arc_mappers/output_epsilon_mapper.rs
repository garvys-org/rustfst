use crate::algorithms::ArcMapper;
use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Mapper that converts all output symbols to epsilon.
pub struct OutputEpsilonMapper {}

impl<S: Semiring> ArcMapper<S> for OutputEpsilonMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        arc.olabel = EPS_LABEL;
    }

    fn final_weight_map(&mut self, _weight: &mut S) {}
}
