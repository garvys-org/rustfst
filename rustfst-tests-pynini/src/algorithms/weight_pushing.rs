use failure::Fallible;

use rustfst::algorithms::{push_weights, ReweightType};
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;

use crate::TestData;

pub fn test_weight_pushing_initial<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Weight pushing initial
    let mut fst_weight_push_initial = test_data.raw.clone();
    push_weights(
        &mut fst_weight_push_initial,
        ReweightType::ReweightToInitial,
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

pub fn test_weight_pushing_final<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Weight pushing final
    let mut fst_weight_push_final = test_data.raw.clone();
    push_weights(&mut fst_weight_push_final, ReweightType::ReweightToFinal)?;
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
