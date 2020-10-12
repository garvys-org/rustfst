use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::closure::{closure, ClosureFst, ClosureType};
use crate::algorithms::fst_convert_from_ref;
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleStaticLazyOperationResult {
    result_static_path: String,
    result_lazy_path: String,
}

pub struct SimpleStaticLazyTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub result_static: F,
    pub result_lazy: F,
    w: PhantomData<W>,
}

impl SimpleStaticLazyOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> SimpleStaticLazyTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        SimpleStaticLazyTestData {
            result_static: F::read(dir_path.as_ref().join(&self.result_static_path)).unwrap(),
            result_lazy: F::read(dir_path.as_ref().join(&self.result_lazy_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_closure_plus<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let closure_test_data = &test_data.closure_plus;
    let mut fst_res_static = test_data.raw.clone();
    closure(&mut fst_res_static, ClosureType::ClosurePlus);
    test_eq_fst(
        &closure_test_data.result_static,
        &fst_res_static,
        "Closure plus",
    );
    Ok(())
}

pub fn test_closure_star<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let closure_test_data = &test_data.closure_star;
    let mut fst_res_static = test_data.raw.clone();
    closure(&mut fst_res_static, ClosureType::ClosureStar);
    test_eq_fst(
        &closure_test_data.result_static,
        &fst_res_static,
        "Closure star",
    );
    Ok(())
}

pub fn test_closure_plus_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let closure_test_data = &test_data.closure_plus;
    let closure_lazy_fst_openfst = &closure_test_data.result_lazy;
    let closure_lazy_fst: VectorFst<_> = fst_convert_from_ref(&ClosureFst::new(
        test_data.raw.clone(),
        ClosureType::ClosurePlus,
    )?);

    test_eq_fst(
        closure_lazy_fst_openfst,
        &closure_lazy_fst,
        "Closure plus lazy",
    );
    Ok(())
}

pub fn test_closure_star_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let closure_test_data = &test_data.closure_star;
    let closure_lazy_fst_openfst = &closure_test_data.result_lazy;
    let closure_lazy_fst: VectorFst<_> = fst_convert_from_ref(&ClosureFst::new(
        test_data.raw.clone(),
        ClosureType::ClosureStar,
    )?);

    test_eq_fst(
        closure_lazy_fst_openfst,
        &closure_lazy_fst,
        "Closure star lazy",
    );
    Ok(())
}
