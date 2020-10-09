use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::invert;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_invert<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Invert
    let mut fst_invert = test_data.raw.clone();
    invert(&mut fst_invert);
    test_eq_fst(&test_data.invert, &fst_invert, "Invert");
    Ok(())
}
