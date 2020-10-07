use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::optimize;
use crate::fst_traits::{MutableFst, SerializableFst, AllocableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize, WeaklyDivisibleSemiring};
use crate::tests_openfst::macros::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_optimize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
    where
        F: MutableFst<W> + Display + SerializableFst<W> + AllocableFst<W>,
        W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let mut fst_optimize = test_data.raw.clone();
    optimize(&mut fst_optimize)?;

    println!("Result optimize = ");
    println!("{}", &fst_optimize);

    test_eq_fst(&test_data.optimize, &fst_optimize, "Optimize");

    Ok(())
}
