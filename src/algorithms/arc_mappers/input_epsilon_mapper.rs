use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Mapper that converts all input symbols to epsilon.
pub struct InputEpsilonMapper {}

impl<S: Semiring> ArcMapper<S> for InputEpsilonMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        arc.ilabel = EPS_LABEL;
    }

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<S>) {}

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
