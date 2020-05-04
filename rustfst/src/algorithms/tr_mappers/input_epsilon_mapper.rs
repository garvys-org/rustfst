use anyhow::Result;

use crate::algorithms::{TrMapper, FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;
use crate::EPS_LABEL;

/// Mapper that converts all input symbols to epsilon.
pub struct InputEpsilonMapper {}

impl<S: Semiring> TrMapper<S> for InputEpsilonMapper {
    fn tr_map(&self, arc: &mut Tr<S>) -> Result<()> {
        arc.ilabel = EPS_LABEL;
        Ok(())
    }

    fn final_tr_map(&self, _final_tr: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

tr_mapper_to_weight_convert_mapper!(InputEpsilonMapper);
