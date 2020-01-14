use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{closure_plus, closure_star, concat, union, ConcatFst, UnionFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::{Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClosureOperationResult {
    result_static: String,
    result_dynamic: String,
}

pub struct ClosureTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub result_static: F,
    pub result_dynamic: F,
}

impl ClosureOperationResult {
    pub fn parse<F>(&self) -> ClosureTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        ClosureTestData {
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
        }
    }
}

pub fn test_closure_plus<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_plus;
    let mut fst_res_static = test_data.raw.clone();
    closure_plus(&mut fst_res_static);

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
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let closure_test_data = &test_data.closure_star;
    let mut fst_res_static = test_data.raw.clone();
    closure_star(&mut fst_res_static);

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

//pub fn test_concat_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
//    where
//        W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
//        W::ReverseWeight: 'static,
//{
//    for concat_test_data in &test_data.concat {
//        let concat_dynamic_fst_openfst = &concat_test_data.result_dynamic;
//        let concat_dynamic_fst =
//            ConcatFst::new(test_data.raw.clone(), concat_test_data.fst_2.clone())?;
//
//        compare_fst_static_dynamic(concat_dynamic_fst_openfst, &concat_dynamic_fst)?;
//    }
//    Ok(())
//}
