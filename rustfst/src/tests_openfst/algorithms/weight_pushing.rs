use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::{push_weights, ReweightType};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_weight_pushing_initial<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Weight pushing initial
    let mut fst_weight_push_initial = test_data.raw.clone();
    push_weights(
        &mut fst_weight_push_initial,
        ReweightType::ReweightToInitial,
    )?;
    test_eq_fst(
        &test_data.weight_pushing_initial,
        &fst_weight_push_initial,
        "Weight Pushing initial",
    );
    Ok(())
}

pub fn test_weight_pushing_final<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Weight pushing final
    let mut fst_weight_push_final = test_data.raw.clone();
    push_weights(&mut fst_weight_push_final, ReweightType::ReweightToFinal)?;
    test_eq_fst(
        &test_data.weight_pushing_final,
        &fst_weight_push_final,
        "Weight Pushing final",
    );
    Ok(())
}
