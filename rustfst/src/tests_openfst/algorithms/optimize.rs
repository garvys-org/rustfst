use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::optimize;
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_optimize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + Display + SerializableFst<W> + AllocableFst<W>,
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
    W::ReverseWeight: WeightQuantize,
{
    let mut fst_optimize = test_data.raw.clone();
    optimize(&mut fst_optimize)?;

    test_eq_fst(&test_data.optimize, &fst_optimize, "Optimize");

    Ok(())
}
