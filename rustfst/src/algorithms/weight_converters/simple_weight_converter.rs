use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper that leaves labels and nextstate unchanged and constructs a new weight
/// from the underlying value of the transition weight.
pub struct SimpleWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for SimpleWeightConverter
where
    SI: Semiring,
    SO: Semiring<Type = SI::Type>,
{
    fn tr_map(&mut self, tr: &Tr<SI>) -> Result<Tr<SO>> {
        Ok(Tr::new(
            tr.ilabel,
            tr.olabel,
            SO::new(tr.weight.value().clone()),
            tr.nextstate,
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

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops
    }
}
