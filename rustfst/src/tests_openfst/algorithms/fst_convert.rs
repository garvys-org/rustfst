use std::fmt::Display;

use crate::algorithms::{fst_convert, fst_convert_from_ref};
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

use crate::tests_openfst::utils::test_eq_fst;
use anyhow::Result;

pub fn test_fst_convert<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display + AllocableFst<W>,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Invert
    let fst = test_data.raw.clone();
    let fst_converted_1: F = fst_convert(fst.clone());
    test_eq_fst(&fst, &fst_converted_1, "fst_convert");

    let fst_converted_2: F = fst_convert_from_ref(&fst);
    test_eq_fst(&fst, &fst_converted_2, "fst_convert");

    Ok(())
}
