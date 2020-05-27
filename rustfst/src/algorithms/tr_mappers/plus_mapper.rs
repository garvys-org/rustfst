use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper to add a constant to all weights.
pub struct PlusMapper<W: Semiring> {
    to_add: W,
}

impl<W: Semiring> PlusMapper<W> {
    pub fn new(value: W::Type) -> Self {
        PlusMapper {
            to_add: W::new(value),
        }
    }

    pub fn from_weight(value: W) -> Self {
        PlusMapper { to_add: value }
    }

    pub fn map_weight(&self, weight: &mut W) -> Result<()> {
        weight.plus_assign(&self.to_add)
    }
}

impl<S: Semiring> TrMapper<S> for PlusMapper<S> {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        self.map_weight(&mut tr.weight)
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()> {
        self.map_weight(&mut final_tr.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops & FstProperties::weight_invariant_properties()
    }
}

tr_mapper_to_weight_convert_mapper!(PlusMapper<S>);
