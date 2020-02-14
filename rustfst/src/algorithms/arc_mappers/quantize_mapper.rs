use failure::Fallible;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::{Semiring, WeightQuantize};
use crate::Arc;
use crate::KDELTA;

/// Mapper to quantize all weights.
pub struct QuantizeMapper {}

pub fn map_weight<W: WeightQuantize>(weight: &mut W) -> Fallible<()> {
    weight.quantize_assign(KDELTA)
}

impl<S: WeightQuantize + Semiring> ArcMapper<S> for QuantizeMapper {
    fn arc_map(&self, arc: &mut Arc<S>) -> Fallible<()> {
        map_weight(&mut arc.weight)
    }

    fn final_arc_map(&self, final_arc: &mut FinalArc<S>) -> Fallible<()> {
        map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

impl<S> WeightConverter<S, S> for QuantizeMapper
where
    S: WeightQuantize,
{
    arc_mapper_to_weight_convert_mapper_methods!(S);
}
