use failure::Fallible;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Mapper that converts all input symbols to epsilon.
pub struct InputEpsilonMapper {}

impl<S: Semiring> ArcMapper<S> for InputEpsilonMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) -> Fallible<()> {
        arc.ilabel = EPS_LABEL;
        Ok(())
    }

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<S>) -> Fallible<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(InputEpsilonMapper);
