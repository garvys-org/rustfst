use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::{push_weights, ReweightType};
use crate::fst_traits::{CoreFst, MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::tests_openfst::FstTestData;

pub fn test_weight_pushing_initial<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeaklyDivisibleSemiring + 'static,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    // Weight pushing initial
    let mut fst_weight_push_initial = test_data.raw.clone();
    push_weights(
        &mut fst_weight_push_initial,
        ReweightType::ReweightToInitial,
        false,
    )?;
    assert_eq!(
        test_data.weight_pushing_initial,
        fst_weight_push_initial,
        "{}",
        error_message_fst!(
            test_data.weight_pushing_initial,
            fst_weight_push_initial,
            "Weight Pushing initial"
        )
    );
    Ok(())
}

pub fn test_weight_pushing_final<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeaklyDivisibleSemiring + 'static,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    // Weight pushing final
    let mut fst_weight_push_final = test_data.raw.clone();
    push_weights(
        &mut fst_weight_push_final,
        ReweightType::ReweightToFinal,
        false,
    )?;
    assert_eq!(
        test_data.weight_pushing_final,
        fst_weight_push_final,
        "{}",
        error_message_fst!(
            test_data.weight_pushing_final,
            fst_weight_push_final,
            "Weight Pushing final"
        )
    );
    Ok(())
}
