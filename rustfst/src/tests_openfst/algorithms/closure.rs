use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{closure, ClosureFst, ClosureType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClosureOperationResult {
    result_static: String,
    result_dynamic: String,
}

pub struct ClosureTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub result_static: F,
    pub result_dynamic: F,
}

impl ClosureOperationResult {
    pub fn parse<F>(&self) -> ClosureTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ClosureTestData {
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
        }
    }
}

pub fn test_closure_plus<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_plus;
    let mut fst_res_static = test_data.raw.clone();
    closure(&mut fst_res_static, ClosureType::ClosurePlus);

    assert_eq!(
        closure_test_data.result_static,
        fst_res_static,
        "{}",
        error_message_fst!(
            closure_test_data.result_static,
            fst_res_static,
            format!("Closure plus failed")
        )
    );
    Ok(())
}

pub fn test_closure_star<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_star;
    let mut fst_res_static = test_data.raw.clone();
    closure(&mut fst_res_static, ClosureType::ClosureStar);

    assert_eq!(
        closure_test_data.result_static,
        fst_res_static,
        "{}",
        error_message_fst!(
            closure_test_data.result_static,
            fst_res_static,
            format!("Closure star failed")
        )
    );
    Ok(())
}

pub fn test_closure_plus_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_plus;
    let closure_dynamic_fst_openfst = &closure_test_data.result_dynamic;
    let closure_dynamic_fst = ClosureFst::new(test_data.raw.clone(), ClosureType::ClosurePlus)?;

    compare_fst_static_dynamic(closure_dynamic_fst_openfst, &closure_dynamic_fst)?;
    Ok(())
}

pub fn test_closure_star_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_star;
    let closure_dynamic_fst_openfst = &closure_test_data.result_dynamic;
    let closure_dynamic_fst = ClosureFst::new(test_data.raw.clone(), ClosureType::ClosureStar)?;

    compare_fst_static_dynamic(closure_dynamic_fst_openfst, &closure_dynamic_fst)?;
    Ok(())
}
