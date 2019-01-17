use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::{Semiring, WeightQuantize};
use crate::Arc;
use crate::KDELTA;

/// Mapper to quantize all weights.
pub struct QuantizeMapper {}

pub fn map_weight<W: WeightQuantize>(weight: &mut W) {
    weight.quantize_assign(KDELTA);
}

impl<S: WeightQuantize + Semiring> ArcMapper<S> for QuantizeMapper {
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
