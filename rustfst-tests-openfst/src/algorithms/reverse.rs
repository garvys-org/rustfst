use failure::Fallible;

use rustfst::algorithms::{isomorphic, reverse, weight_convert};
use rustfst::algorithms::weight_converters::SimpleWeightConverter;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{MutableFst, CoreFst};
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;

use crate::TestData;
use rustfst::algorithms::WeightConverter;
use rustfst::Arc;
use rustfst::algorithms::FinalArc;
use rustfst::algorithms::MapFinalAction;

pub struct ReverseWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for ReverseWeightConverter
    where
        SI: Semiring,
        SO: Semiring,
{
    fn arc_map(&mut self, arc: &Arc<SI>) -> Fallible<Arc<SO>> {
        let w = &arc.weight;
        let rw = unsafe {std::mem::transmute::<
            &SI, &SO,
        >(w).clone()};

        Ok(Arc::new(
            arc.ilabel,
            arc.olabel,
            rw,
            arc.nextstate,
        ))
    }

    fn final_arc_map(&mut self, final_arc: &FinalArc<SI>) -> Fallible<FinalArc<SO>> {
        let w = &final_arc.weight;
        let rw = unsafe {std::mem::transmute::<
            &SI, &SO,
        >(w).clone() };
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
//     Reverse
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    let mut mapper = ReverseWeightConverter {};
    let fst_reverse_2 : VectorFst<F::W> = weight_convert(&fst_reverse, &mut mapper)?;
    assert!(
        isomorphic(&test_data.reverse, &fst_reverse_2)?,
        "{}",
        error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
    );
    Ok(())
}
