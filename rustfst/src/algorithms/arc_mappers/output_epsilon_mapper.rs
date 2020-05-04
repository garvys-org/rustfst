use anyhow::Result;

use crate::algorithms::{TrMapper, FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;
use crate::EPS_LABEL;

/// Mapper that converts all output symbols to epsilon.
pub struct OutputEpsilonMapper {}

impl<S: Semiring> TrMapper<S> for OutputEpsilonMapper {
    fn arc_map(&self, arc: &mut Tr<S>) -> Result<()> {
        arc.olabel = EPS_LABEL;
        Ok(())
    }

    fn final_arc_map(&self, _final_arc: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(OutputEpsilonMapper);
