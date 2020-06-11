use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::Tr;
use crate::EPS_LABEL;

/// Mapper that converts all input symbols to epsilon.
pub struct InputEpsilonMapper {}

impl<S: Semiring> TrMapper<S> for InputEpsilonMapper {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        tr.ilabel = EPS_LABEL;
        Ok(())
    }

    fn final_tr_map(&self, _final_tr: &mut FinalTr<S>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        (inprops & FstProperties::set_arc_properties())
            | FstProperties::I_EPSILONS
            | FstProperties::I_LABEL_SORTED
    }
}

tr_mapper_to_weight_convert_mapper!(InputEpsilonMapper);
