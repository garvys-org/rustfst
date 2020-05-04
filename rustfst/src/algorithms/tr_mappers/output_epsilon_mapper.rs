use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;
use crate::EPS_LABEL;

/// Mapper that converts all output symbols to epsilon.
pub struct OutputEpsilonMapper {}

impl<S: Semiring> TrMapper<S> for OutputEpsilonMapper {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        tr.olabel = EPS_LABEL;
        Ok(())
    }

    fn final_tr_map(&self, _final_tr: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

tr_mapper_to_weight_convert_mapper!(OutputEpsilonMapper);
