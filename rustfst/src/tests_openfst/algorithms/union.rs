use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::union::{union, UnionFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::lazy_fst::compare_fst_static_lazy;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnionOperationResult {
    fst_2: String,
    result_static: String,
    result_lazy: String,
}

pub struct UnionTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_lazy: F,
    w: PhantomData<W>,
}

impl UnionOperationResult {
    pub fn parse<W, F>(&self) -> UnionTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        UnionTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_lazy: F::from_text_string(self.result_lazy.as_str()).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_union<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for union_test_data in &test_data.union {
        let mut fst_res_static = test_data.raw.clone();
        union(&mut fst_res_static, &union_test_data.fst_2)?;

        assert_eq!(
            union_test_data.result_static,
            fst_res_static,
            "{}",
            error_message_fst!(
                union_test_data.result_static,
                fst_res_static,
                format!("Union failed")
            )
        );
    }
    Ok(())
}

pub fn test_union_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for union_test_data in &test_data.union {
        let union_lazy_fst_openfst = &union_test_data.result_lazy;
        let union_lazy_fst = UnionFst::new(test_data.raw.clone(), union_test_data.fst_2.clone())?;

        compare_fst_static_lazy(union_lazy_fst_openfst, &union_lazy_fst)?;
    }
    Ok(())
}
