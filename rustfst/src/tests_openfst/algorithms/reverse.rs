use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::FinalTr;
use crate::algorithms::MapFinalAction;
use crate::algorithms::WeightConverter;
use crate::algorithms::{reverse, weight_convert};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, CoreFst, MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::Tr;

use crate::tests_openfst::FstTestData;

pub struct ReverseWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for ReverseWeightConverter
where
    SI: Semiring,
    SO: Semiring,
{
    fn tr_map(&mut self, tr: &Tr<SI>) -> Result<Tr<SO>> {
        let w = &tr.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };

        Ok(Tr::new(tr.ilabel, tr.olabel, rw, tr.nextstate))
    }

    fn final_tr_map(&mut self, final_tr: &FinalTr<SI>) -> Result<FinalTr<SO>> {
        let w = &final_tr.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };
        Ok(FinalTr {
            ilabel: final_tr.ilabel,
            olabel: final_tr.olabel,
            weight: rw,
        })
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

pub fn test_reverse<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + AllocableFst + Display,
    F::W: 'static + SerializableSemiring + WeaklyDivisibleSemiring,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: SerializableSemiring,
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
