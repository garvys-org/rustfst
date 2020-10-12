use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::FinalTr;
use crate::algorithms::MapFinalAction;
use crate::algorithms::WeightConverter;
use crate::algorithms::{reverse, weight_convert};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::{Semiring, SerializableSemiring};
use crate::semirings::{WeaklyDivisibleSemiring, WeightQuantize};
use crate::Tr;

use crate::fst_properties::FstProperties;
use crate::tests_openfst::utils::test_eq_fst;
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

    fn properties(&self, iprops: FstProperties) -> FstProperties {
        iprops
    }
}

pub fn test_reverse<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + AllocableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: SerializableSemiring,
{
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    let mut mapper = ReverseWeightConverter {};
    let fst_reverse_2: F = weight_convert(&fst_reverse, &mut mapper)?;

    test_eq_fst(&test_data.reverse, &fst_reverse_2, "Reverse");
    Ok(())
}
