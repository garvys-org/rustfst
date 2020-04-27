use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::{concat, ConcatFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConcatOperationResult {
    fst_2: String,
    result_static: String,
    result_dynamic: String,
}

pub struct ConcatTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_dynamic: F,
}

impl ConcatOperationResult {
    pub fn parse<F>(&self) -> ConcatTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ConcatTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
        }
    }
}

pub fn test_concat<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for concat_test_data in &test_data.concat {
        let mut fst_res_static = test_data.raw.clone();
        concat(&mut fst_res_static, &concat_test_data.fst_2)?;

        assert_eq!(
            concat_test_data.result_static,
            fst_res_static,
            "{}",
            error_message_fst!(
                concat_test_data.result_static,
                fst_res_static,
                format!("Concat failed")
            )
        );
    }
    Ok(())
}

pub fn test_concat_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for concat_test_data in &test_data.concat {
        let concat_dynamic_fst_openfst = &concat_test_data.result_dynamic;
        let concat_dynamic_fst =
            ConcatFst::new(test_data.raw.clone(), concat_test_data.fst_2.clone())?;

        compare_fst_static_dynamic(concat_dynamic_fst_openfst, &concat_dynamic_fst)?;
    }
    Ok(())
}
