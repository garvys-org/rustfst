use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::concat::{concat, ConcatFst};
use crate::algorithms::fst_convert_from_ref;
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConcatOperationResult {
    fst_2_path: String,
    result_static_path: String,
    result_lazy_path: String,
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
    pub fn parse<W, F, P>(&self, dir_path: P) -> ConcatTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        ConcatTestData {
            fst_2: F::read(dir_path.as_ref().join(&self.fst_2_path)).unwrap(),
            result_static: F::read(dir_path.as_ref().join(&self.result_static_path)).unwrap(),
            result_lazy: F::read(dir_path.as_ref().join(&self.result_lazy_path)).unwrap(),
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

        test_eq_fst(&concat_test_data.result_static, &fst_res_static, "Concat");
    }
    Ok(())
}

pub fn test_concat_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for concat_test_data in &test_data.concat {
        let concat_lazy_fst_openfst = &concat_test_data.result_lazy;
        let concat_lazy_fst: VectorFst<_> = fst_convert_from_ref(&ConcatFst::new(
            test_data.raw.clone(),
            concat_test_data.fst_2.clone(),
        )?);

        test_eq_fst(concat_lazy_fst_openfst, &concat_lazy_fst, "Concat lazy");
    }
    Ok(())
}
