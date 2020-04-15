use anyhow::Result;

use crate::algorithms::{FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper that leaves labels and nextstate unchanged and constructs a new weight
/// from the underlying value of the arc weight.
pub struct SimpleWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for SimpleWeightConverter
where
    SI: Semiring,
    SO: Semiring<Type = SI::Type>,
{
    fn arc_map(&mut self, arc: &Arc<SI>) -> Result<Arc<SO>> {
        Ok(Arc::new(
            arc.ilabel,
            arc.olabel,
            SO::new(arc.weight.value().clone()),
            arc.nextstate,
        ))
    }

    fn final_arc_map(&mut self, final_arc: &FinalArc<SI>) -> Result<FinalArc<SO>> {
        Ok(FinalArc {
            ilabel: final_arc.ilabel,
            olabel: final_arc.olabel,
            weight: SO::new(final_arc.weight.value().clone()),
        })
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
