use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{union, UnionFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnionOperationResult {
    fst_2: String,
    result_static: String,
    result_dynamic: String,
}

pub struct UnionTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_dynamic: F,
}

impl UnionOperationResult {
    pub fn parse<F>(&self) -> UnionTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        UnionTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
        }
    }
}

pub fn test_union<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
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

pub fn test_union_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for union_test_data in &test_data.union {
        let union_dynamic_fst_openfst = &union_test_data.result_dynamic;
        let union_dynamic_fst =
            UnionFst::new(test_data.raw.clone(), union_test_data.fst_2.clone())?;

        compare_fst_static_dynamic(union_dynamic_fst_openfst, &union_dynamic_fst)?;
    }
    Ok(())
}
