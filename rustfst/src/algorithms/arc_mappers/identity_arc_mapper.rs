use anyhow::Result;

use crate::algorithms::{TrMapper, FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper that returns its input.
pub struct IdentityTrMapper {}

impl<S: Semiring> TrMapper<S> for IdentityTrMapper {
    fn arc_map(&self, _arc_to_map: &mut Tr<S>) -> Result<()> {
        Ok(())
    }

    fn final_arc_map(&self, _final_arc: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(IdentityTrMapper);
