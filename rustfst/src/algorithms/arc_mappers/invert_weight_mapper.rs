use anyhow::Result;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::{DivideType, WeaklyDivisibleSemiring};
use crate::Arc;

/// Mapper to reciprocate all non-Zero() weights.
pub struct InvertWeightMapper {}

#[inline]
pub fn map_weight<W: WeaklyDivisibleSemiring>(weight: &mut W) -> Result<()> {
    weight.set_value(W::one().divide(weight, DivideType::DivideAny)?.take_value());
    Ok(())
}

impl<S: WeaklyDivisibleSemiring> ArcMapper<S> for InvertWeightMapper {
    fn arc_map(&self, arc: &mut Arc<S>) -> Result<()> {
        map_weight(&mut arc.weight)
    }

    fn final_arc_map(&self, final_arc: &mut FinalArc<S>) -> Result<()> {
        map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

impl<S> WeightConverter<S, S> for InvertWeightMapper
where
    S: WeaklyDivisibleSemiring,
{
    arc_mapper_to_weight_convert_mapper_methods!(S);
}
