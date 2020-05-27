use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper that returns its input.
pub struct IdentityTrMapper {}

impl<S: Semiring> TrMapper<S> for IdentityTrMapper {
    fn tr_map(&self, _tr_to_map: &mut Tr<S>) -> Result<()> {
        Ok(())
    }

    fn final_tr_map(&self, _final_tr: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops
    }
}

tr_mapper_to_weight_convert_mapper!(IdentityTrMapper);
