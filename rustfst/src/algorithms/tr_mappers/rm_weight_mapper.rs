use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper to map all non-Zero() weights to One().
pub struct RmWeightMapper {}

pub fn map_weight<W: Semiring>(weight: &mut W) {
    if !weight.is_zero() {
        weight.set_value(W::one().take_value())
    }
}

impl<S: Semiring> TrMapper<S> for RmWeightMapper {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        map_weight(&mut tr.weight);
        Ok(())
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()> {
        map_weight(&mut final_tr.weight);
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

tr_mapper_to_weight_convert_mapper!(RmWeightMapper);
