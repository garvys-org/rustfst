use failure::Fallible;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper that returns its input.
pub struct IdentityArcMapper {}

impl<S: Semiring> ArcMapper<S> for IdentityArcMapper {
    fn arc_map(&self, _arc_to_map: &mut Arc<S>) -> Fallible<()> {
        Ok(())
    }

    fn final_arc_map(&self, _final_arc: &mut FinalArc<S>) -> Fallible<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(IdentityArcMapper);
