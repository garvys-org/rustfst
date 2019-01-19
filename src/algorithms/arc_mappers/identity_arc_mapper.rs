use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper that returns its input.
pub struct IdentityArcMapper {}

impl<S: Semiring> ArcMapper<S> for IdentityArcMapper {
    fn arc_map(&mut self, _arc_to_map: &mut Arc<S>) {}

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<S>) {}

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
