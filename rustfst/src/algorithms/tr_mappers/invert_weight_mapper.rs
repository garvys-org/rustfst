use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::{DivideType, WeaklyDivisibleSemiring};
use crate::Tr;

/// Mapper to reciprocate all non-Zero() weights.
pub struct InvertWeightMapper {}

#[inline]
pub fn map_weight<W: WeaklyDivisibleSemiring>(weight: &mut W) -> Result<()> {
    weight.set_value(W::one().divide(weight, DivideType::DivideAny)?.take_value());
    Ok(())
}

impl<S: WeaklyDivisibleSemiring> TrMapper<S> for InvertWeightMapper {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        map_weight(&mut tr.weight)
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()> {
        map_weight(&mut final_tr.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops & FstProperties::weight_invariant_properties()
    }
}

impl<S> WeightConverter<S, S> for InvertWeightMapper
where
    S: WeaklyDivisibleSemiring,
{
    tr_mapper_to_weight_convert_mapper_methods!(S);
}
