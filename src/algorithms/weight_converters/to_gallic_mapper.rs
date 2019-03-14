use crate::algorithms::{WeightConverter, MapFinalAction, FinalArc};
use crate::{Arc, EPS_LABEL};
use crate::semirings::{Semiring, GallicWeightLeft, ProductWeight,StringWeightLeft, StringWeightVariant};

// Mapper from A to GallicArc<A>.
pub struct ToGallicConverter {}

impl<S> WeightConverter<S, GallicWeightLeft<S>> for ToGallicConverter
where
    S: Semiring
{
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<GallicWeightLeft<S>> {
        if arc.olabel == EPS_LABEL {
            let mut w = GallicWeightLeft::one();
            w.weight.set_value2(arc.weight.clone());
            Arc::new(arc.ilabel, arc.ilabel, w, arc.nextstate)
        } else {
            let mut w = GallicWeightLeft::one();
            w.weight.set_value1(StringWeightLeft::new(StringWeightVariant::Labels(vec![arc.olabel])));
            w.weight.set_value2(arc.weight.clone());
            Arc::new(arc.ilabel, arc.ilabel, w, arc.nextstate)
        }
    }

    fn final_arc_map(&mut self, final_arc: &FinalArc<S>) -> FinalArc<GallicWeightLeft<S>> {
        let mut w = GallicWeightLeft::one();
        w.weight.set_value2(final_arc.weight.clone());
        FinalArc {
            ilabel: EPS_LABEL,
            olabel: EPS_LABEL,
            weight: w
        }
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
