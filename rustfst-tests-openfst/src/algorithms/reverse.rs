use failure::Fallible;

use rustfst::algorithms::FinalArc;
use rustfst::algorithms::MapFinalAction;
use rustfst::algorithms::WeightConverter;
use rustfst::algorithms::{reverse, weight_convert};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;
use rustfst::Arc;

use crate::TestData;

pub struct ReverseWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for ReverseWeightConverter
where
    SI: Semiring,
    SO: Semiring,
{
    fn arc_map(&mut self, arc: &Arc<SI>) -> Fallible<Arc<SO>> {
        let w = &arc.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };

        Ok(Arc::new(arc.ilabel, arc.olabel, rw, arc.nextstate))
    }

    fn final_arc_map(&mut self, final_arc: &FinalArc<SI>) -> Fallible<FinalArc<SO>> {
        let w = &final_arc.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };
        Ok(FinalArc {
            ilabel: final_arc.ilabel,
            olabel: final_arc.olabel,
            weight: rw,
        })
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

pub fn test_reverse<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    let mut mapper = ReverseWeightConverter {};
    let fst_reverse_2: F = weight_convert(&fst_reverse, &mut mapper)?;
    assert_eq!(
        test_data.reverse,
        fst_reverse_2,
        "{}",
        error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
    );
    Ok(())
}
