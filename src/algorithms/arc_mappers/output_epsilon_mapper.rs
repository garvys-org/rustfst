use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Mapper that converts all output symbols to epsilon.
pub struct OutputEpsilonMapper {}

impl<S: Semiring> ArcMapper<S> for OutputEpsilonMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        arc.olabel = EPS_LABEL;
    }

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<S>) {}

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
