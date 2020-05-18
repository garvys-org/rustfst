use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::concat::{concat, ConcatFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::lazy_fst::compare_fst_static_lazy;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConcatOperationResult {
    fst_2: String,
    result_static: String,
    result_lazy: String,
}

pub struct ConcatTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_lazy: F,
    w: PhantomData<W>,
}

impl ConcatOperationResult {
    pub fn parse<W, F>(&self) -> ConcatTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        ConcatTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_lazy: F::from_text_string(self.result_lazy.as_str()).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_concat<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
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

pub fn test_concat_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for concat_test_data in &test_data.concat {
        let concat_lazy_fst_openfst = &concat_test_data.result_lazy;
        let concat_lazy_fst =
            ConcatFst::new(test_data.raw.clone(), concat_test_data.fst_2.clone())?;

        compare_fst_static_lazy(concat_lazy_fst_openfst, &concat_lazy_fst)?;
    }
    Ok(())
}
