use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::Arc;

/// Mapper to reciprocate all non-Zero() weights.
pub struct InvertWeightMapper {}

pub fn map_weight<W: WeaklyDivisibleSemiring>(weight: &mut W) {
    weight.inverse_assign();
}

impl<S: WeaklyDivisibleSemiring> ArcMapper<S> for InvertWeightMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        map_weight(&mut arc.weight)
    }

    fn final_arc_map(&mut self, final_arc: &mut FinalArc<S>) {
        map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
