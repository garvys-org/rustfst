use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper that leaves labels and nextstate unchanged and constructs a new weight
/// from the underlying value of the arc weight.
pub struct SimpleWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for SimpleWeightConverter
where
    SI: Semiring,
    SO: Semiring<Type = SI::Type>,
{
    fn tr_map(&mut self, arc: &Tr<SI>) -> Result<Tr<SO>> {
        Ok(Tr::new(
            arc.ilabel,
            arc.olabel,
            SO::new(arc.weight.value().clone()),
            arc.nextstate,
        ))
    }

    fn final_tr_map(&mut self, final_tr: &FinalTr<SI>) -> Result<FinalTr<SO>> {
        Ok(FinalTr {
            ilabel: final_tr.ilabel,
            olabel: final_tr.olabel,
            weight: SO::new(final_tr.weight.value().clone()),
        })
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
